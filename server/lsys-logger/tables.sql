CREATE TABLE `yaf_change_logs` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `log_type` varchar(32) NOT NULL COMMENT '日志类型',
    `log_data` text NOT NULL COMMENT '日志数据',
    `message` varchar(255) DEFAULT '' NULL COMMENT '消息',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '操作记录用户ID',
    `source_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '相关操作记录ID',
    `add_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '操作用户ID',
    `user_ip` varchar(40) NOT NULL DEFAULT '' COMMENT '操作者IP',
    `request_id` varchar(32) NOT NULL DEFAULT '' COMMENT '请求id',
    `request_user_agent` varchar(254) NOT NULL DEFAULT '' COMMENT '请求UA',
    `add_time` bigint unsigned NOT NULL COMMENT '添加时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '操作日志';