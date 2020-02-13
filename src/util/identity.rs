use crate::error::{Detail, Error, Kind};
use actix_web::cookie::{Cookie, CookieJar, Key};
use actix_web::dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::{header, HeaderValue};
use actix_web::{Error as ActixError, FromRequest, HttpMessage, HttpRequest};
use deadpool_redis::{cmd, Pool as RedisPool};
use failure::_core::cell::RefCell;
use futures::future::LocalBoxFuture;
use futures::task::{Context, Poll};
use futures::{future, FutureExt};
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::iter;
use std::rc::Rc;
use time::Duration;

/// 自定义的身份标识
pub struct Identity(HttpRequest);

// 实现了 `FromRequest` 就可以出现在 router 函数的参数列表中
impl FromRequest for Identity {
    type Error = ActixError;
    type Future = future::Ready<Result<Identity, ActixError>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        future::ok(Identity(req.clone()))
    }
}

impl Identity {
    const DEFAULT_DURATION_SEC: i64 = 24 * 3600;

    pub fn default_ttl() -> Duration {
        Duration::seconds(Self::DEFAULT_DURATION_SEC)
    }

    /// 获取身份标识
    ///
    /// TODO: Deserialize<'de> 玩不转，只能用 DeserializeOwned，是跟 extensions() 返回值的声明周期有关，还是我用法不对
    pub fn get<T: Serialize + DeserializeOwned>(&self) -> Option<T> {
        if let Some(cache) = self.0.extensions().get::<IdentityCache>() {
            match cache {
                IdentityCache::User { identity, .. } => serde_json::from_str(identity).ok(),
                IdentityCache::Guest { .. } => None,
            }
        } else {
            None
        }
    }

    /// 判断当前用户是否已登录
    pub fn is_user(&self) -> bool {
        if let Some(cache) = self.0.extensions().get::<IdentityCache>() {
            match cache {
                IdentityCache::User { .. } => true,
                IdentityCache::Guest { .. } => false,
            }
        } else {
            false
        }
    }

    /// 登录
    #[allow(dead_code)]
    pub fn sign_in_ttl<T: Serialize + DeserializeOwned>(
        &self,
        id: T,
        ttl: Duration,
    ) -> Result<(), serde_json::Error> {
        self.0.extensions_mut().insert(IdentityCache::Guest {
            action: Some(SignIn {
                identity: serde_json::to_string(&id)?,
                ttl: Some(ttl),
            }),
        });

        Ok(())
    }

    /// 登录
    pub fn sign_in<T: Serialize + DeserializeOwned>(&self, id: T) -> Result<(), Error> {
        self.0.extensions_mut().insert(IdentityCache::Guest {
            action: Some(SignIn {
                identity: serde_json::to_string(&id)
                    .map_err(|e| Kind::DATA_FORMAT.with_detail(Detail::from(e)))?,
                ttl: Some(Self::default_ttl()),
            }),
        });

        Ok(())
    }

    /// 登出
    pub fn sign_out(&self) {
        if let Some(cache) = self.0.extensions_mut().get_mut::<IdentityCache>() {
            if let IdentityCache::User { action, .. } = cache {
                action.replace(SignOut);
            }
        }
    }
}

/// 缓存当前身份标识及下一步操作，放在 HttpRequest 的 Extension 中
enum IdentityCache {
    User {
        identity: String,
        action: Option<SignOut>,
    },
    Guest {
        action: Option<SignIn>,
    },
}

/// 登录
struct SignIn {
    identity: String,
    ttl: Option<Duration>,
}

/// 登出
struct SignOut;

struct IdentityInner {
    key: Key,
    secure: bool,
    name: String,
    default_ttl: Duration,
}

/// 身份标识中间件工厂
pub struct IdentityFactory {
    inner: Rc<IdentityInner>,
    pool: RedisPool,
}

impl IdentityFactory {
    /// 使用 Redis 线程池创建一个身份标识中间件工厂
    pub fn new(key: &[u8], redis_pool: RedisPool) -> Self {
        let key: Vec<u8> = key.iter().chain([1, 0, 0, 0].iter()).cloned().collect();
        Self {
            inner: Rc::new(IdentityInner {
                key: Key::from_master(&key),
                secure: false,
                name: "identity-cookie".into(),
                default_ttl: Identity::default_ttl(),
            }),
            pool: redis_pool,
        }
    }

    pub fn name<T: Into<String>>(mut self, value: T) -> IdentityFactory {
        Rc::get_mut(&mut self.inner).unwrap().name = value.into();
        self
    }

    #[allow(dead_code)]
    pub fn default_ttl(mut self, ttl: Duration) -> IdentityFactory {
        Rc::get_mut(&mut self.inner).unwrap().default_ttl = ttl;
        self
    }

    #[allow(dead_code)]
    pub fn secure(mut self, secure: bool) -> IdentityFactory {
        Rc::get_mut(&mut self.inner).unwrap().secure = secure;
        self
    }
}

impl<S, B> Transform<S> for IdentityFactory
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>
        + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Transform = IdentityMiddleware<S>;
    type InitError = ();
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(IdentityMiddleware {
            service: Rc::new(RefCell::new(service)),
            inner: self.inner.clone(),
            pool: self.pool.clone(),
        })
    }
}

const IDENTITY_KEY_PREFIX: &str = "user:identity:";
const IDENTITY_KEY_RAND_LEN: usize = 32;

fn make_redis_key(token: &str) -> String {
    let mut redis_key = String::with_capacity(IDENTITY_KEY_PREFIX.len() + IDENTITY_KEY_RAND_LEN);
    redis_key += IDENTITY_KEY_PREFIX;
    redis_key += token;
    redis_key
}

pub struct IdentityMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<RefCell<S>>,
    inner: Rc<IdentityInner>,
    pool: RedisPool,
}

impl<S, B> Service for IdentityMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>
        + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let mut svc = self.service.clone();
        let pool = self.pool.clone();
        let inner = self.inner.clone();

        let token = if let Some(cookie) = req.cookie(&inner.name) {
            let mut jar = CookieJar::new();
            jar.add_original(cookie.clone());
            jar.signed(&inner.key)
                .get(&inner.name)
                .map(|c| c.value().to_owned())
        } else {
            None
        };

        Box::pin(
            async move {
                // 如果 cookie 中存在 key，尝试从 redis 中取出
                if let Some(token) = &token {
                    let mut conn = pool
                        .get()
                        .await
                        .map_err(Error::from)
                        .map_err(ActixError::from)?;
                    let id: Option<String> = cmd("GET")
                        .arg(make_redis_key(token))
                        .query_async(&mut conn)
                        .await
                        .map_err(Error::from)
                        .map_err(ActixError::from)?;

                    if let Some(identity) = id {
                        // 如果取成功了，在 HttpRequest 的 extensions 中插入用户身份标识
                        req.extensions_mut().insert(IdentityCache::User {
                            identity,
                            action: None,
                        });
                    } else {
                        // 如果取失败了，证明是游客
                        req.extensions_mut()
                            .insert(IdentityCache::Guest { action: None });
                    }
                } else {
                    // 如果 cookie 中没有 key，证明是游客
                    req.extensions_mut()
                        .insert(IdentityCache::Guest { action: None });
                }

                // 调用下一个 service，包括 router
                let mut response: ServiceResponse<B> = svc.call(req).await?;

                let mut jar = CookieJar::new();
                let key = &inner.key;

                // 根据 extensions 里 IdentityCache 对象的 action 的变化执行登录/登出
                if let Some(cache) = response.request().extensions().get::<IdentityCache>() {
                    match cache {
                        IdentityCache::User {
                            action: Some(_), ..
                        } => {
                            // 如果是用户且有动作就尝试登出
                            if let Some(token) = &token {
                                let mut conn = pool
                                    .get()
                                    .await
                                    .map_err(Error::from)
                                    .map_err(ActixError::from)?;
                                cmd("DEL")
                                    .arg(make_redis_key(token))
                                    .execute_async(&mut conn)
                                    .await
                                    .map_err(Error::from)
                                    .map_err(ActixError::from)?;
                            }

                            // 设置cookie立即失效
                            let cookie = Cookie::new(inner.name.clone(), "");
                            jar.add_original(cookie.clone());
                            jar.signed(key).remove(cookie);
                        }
                        IdentityCache::Guest { action: Some(si) } => {
                            // 如果是游客并且有登陆动作
                            let token: String = iter::repeat(())
                                .map(|()| OsRng.sample(Alphanumeric))
                                .take(32)
                                .collect();

                            let ttl = si.ttl.unwrap_or(inner.default_ttl);
                            let mut conn = pool
                                .get()
                                .await
                                .map_err(Error::from)
                                .map_err(ActixError::from)?;
                            cmd("SETEX")
                                .arg(&make_redis_key(&token))
                                .arg(ttl.num_seconds())
                                .arg(&si.identity)
                                .execute_async(&mut conn)
                                .await
                                .map_err(Error::from)
                                .map_err(ActixError::from)?;

                            // 设置 cookie
                            let mut cookie = Cookie::new(inner.name.to_owned(), token);
                            cookie.set_max_age(ttl);
                            cookie.set_secure(inner.secure);
                            cookie.set_http_only(true);
                            jar.signed(key).add(cookie);
                        }
                        _ => {}
                    }
                };

                for cookie in jar.delta() {
                    let val = HeaderValue::from_str(&cookie.to_string())?;
                    response.headers_mut().append(header::SET_COOKIE, val);
                }

                Ok(response)
            }
            .boxed_local(),
        )
    }
}
