-- Add migration script here
create table health_check
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    request_time TIMESTAMP NOT NULL,
    node_name        varchar(255) NOT NULL, -- 节点名称
    status_code        INTEGER NOT NULL, -- 测试状态码
    status_des        varchar(255), -- 测试描述
    delay_ms        INTEGER DEFAULT 0 -- 延迟，>0为正常延迟，-1为超时或服务不可用
);