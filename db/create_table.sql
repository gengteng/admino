-- 行更新时，update_time 字段更新为当前时间
CREATE OR REPLACE FUNCTION update_modified_column()
    RETURNS TRIGGER AS $$
BEGIN
    IF row(NEW.*) IS DISTINCT FROM row(OLD.*) THEN
        NEW.update_time = now();
        RETURN NEW;
    ELSE
        RETURN OLD;
    END IF;
END;
$$ language 'plpgsql';

create type "AuthType" as enum (
    'Username',
    'Phone',
    'Email'
    );

comment on type "AuthType" is '授权类型';

-- 用户表
create table user_info
(
    id bigserial not null
        constraint user_info_pk
            primary key,
    username text not null,
    nickname text not null,
    avatar text,
    gender integer default 0 not null,
    birthday date,
    update_time timestamp default now() not null,
    create_time timestamp default now() not null,
    max_role bigint,
    constraint user_info_username_unique
        unique (username)
);

comment on table user_info is '用户信息表';
comment on column user_info.id is '用户ID';
comment on column user_info.username is '用户名';
comment on column user_info.nickname is '用户昵称';
comment on column user_info.avatar is '头像图片';
comment on column user_info.gender is '性别';
comment on column user_info.birthday is '出生日期';
comment on column user_info.update_time is '更新时间';
comment on column user_info.max_role is '最大角色数';

create trigger user_info_on_update
    before update
    on user_info
    for each row
execute procedure update_modified_column();


-- 用户授权表
create table user_auth
(
    user_id bigint not null
        constraint user_auth_fk_user_info
            references user_info,
    auth_type "AuthType" not null,
    identity varchar(255) not null,
    credential1 varchar(255) not null,
    credential2 varchar(255),
    create_time timestamp default now() not null,
    update_time timestamp default now() not null,
    constraint user_auth_pk
        primary key (user_id, auth_type),
    constraint user_auth_type_identity_unique
        unique (auth_type, identity)
);

comment on table user_auth is '用户授权表';
comment on column user_auth.user_id is '用户ID';
comment on column user_auth.auth_type is '授权方式';
comment on column user_auth.identity is '身份标识';
comment on column user_auth.credential1 is '密码1';
comment on column user_auth.credential2 is '密码2';
comment on column user_auth.create_time is '创建时间';
comment on column user_auth.update_time is '更新时间';

create trigger user_auth_on_update
    before update
    on user_auth
    for each row
execute procedure update_modified_column();

-- 角色表
create table role
(
    id bigserial not null
        constraint role_pk
            primary key,
    name varchar(16) not null,
    max_user bigint,
    max_permission bigint,
    constraint role_name_unique
        unique (name)
);

comment on table role is '角色表';
comment on column role.id is '角色ID';
comment on column role.name is '角色名';

-- 用户角色表
create table user_role
(
    user_id bigint not null
        constraint user_role_fk_user
            references user_info,
    role_id bigint not null
        constraint user_role_fk_role
            references role,
    constraint user_role_pk
        primary key (user_id, role_id)
);

comment on table user_role is '用户角色表';
comment on column user_role.user_id is '用户ID';
comment on column user_role.role_id is '角色ID';

-- 角色继承表
create table role_ext
(
    base_id bigint not null
        constraint role_ext_fk_role_base
            references role,
    derived_id bigint not null
        constraint role_ext_fk_role_derived
            references role,
    constraint role_ext_pk
        primary key (base_id, derived_id)
);

comment on table role_ext is '角色继承表';
comment on column role_ext.base_id is '父角色ID';
comment on column role_ext.derived_id is '派生角色ID';

-- 约束类型
create type "ConstraintType" as enum (
    'Mutex',
    'BaseRequired'
    );

comment on type "ConstraintType" is '约束类型';

-- 角色约束表
create table role_constraint
(
    id bigserial not null
        constraint role_constraints_pk
            primary key,
    constraint_name text not null,
    constraint_type "ConstraintType" not null,
    constraint role_constraint_name_unique
        unique (constraint_name)
);

comment on table role_constraint is '角色约束表';
comment on column role_constraint.constraint_name is '约束名';
comment on column role_constraint.constraint_type is '约束类型';

-- 角色互斥约束表: 用户不能身兼同一约束中的多个角色
create table constraint_mutex
(
    constraint_id bigint not null
        constraint constraint_mutex_fk_role_constraint
            references role_constraint(id),
    role_id bigint not null
        constraint constraint_mutex_fk_role
            references role(id),
    constraint constraint_mutex_pk
        primary key (constraint_id, role_id)
);

comment on table constraint_mutex is '角色互斥约束表';
comment on column constraint_mutex.constraint_id is '约束ID';
comment on column constraint_mutex.role_id is '角色ID';

-- 角色先决条件约束表: 用户想获得某派生的角色(role_id), 必须先获得其父角色
create table constraint_base_required
(
    constraint_id bigint not null
        constraint constraint_mutex_fk_role_constraint
            references role_constraint,
    role_id bigint not null
        constraint constraint_mutex_fk_role
            references role,
    constraint constraint_base_required_pk
        primary key (constraint_id)
);

comment on table constraint_base_required is '角色先决条件约束表';
comment on column constraint_base_required.constraint_id is '约束ID';
comment on column constraint_base_required.role_id is '角色ID';

-- 权限表
create table permission
(
    id bigserial not null
        constraint permission_pk
            primary key,
    permission_name text not null,
    constraint permission_name_unique
        unique (permission_name)
);

comment on table permission is '权限表';
comment on column permission.id is '权限ID';
comment on column permission.permission_name is '权限名';

-- 角色权限表
create table role_permission
(
    role_id bigint not null
        constraint role_perm_fk_role
            references role,
    permission_id bigint not null
        constraint role_perm_fk_permission
            references permission,
    constraint role_perm_pk
        primary key (role_id, permission_id)
);

comment on table role_permission is '角色权限表';
comment on column role_permission.role_id is '角色ID';
comment on column role_permission.permission_id is '权限ID';