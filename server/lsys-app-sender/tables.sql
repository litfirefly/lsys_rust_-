-- ----------- lsys-app-sender  ---------------
CREATE TABLE `yaf_sender_config` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `priority` tinyint DEFAULT 99,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `config_type` tinyint NOT NULL COMMENT '配置类型',
    `config_data` varchar(512) NOT NULL COMMENT '配置数据',
    `status` tinyint NOT NULL COMMENT '启用状态',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户id',
    `change_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送配置,如发送限额等';
CREATE TABLE `yaf_sender_message_cancel` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `sender_body_id` bigint unsigned NOT NULL COMMENT '消息内容ID',
    `sender_message_id` bigint unsigned NOT NULL COMMENT '消息ID',
    `cancel_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '用户id',
    `cancel_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE,
    KEY `message_id` (`sender_message_id`) USING BTREE,
    KEY `sender_type` (`sender_type`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '取消发送列表';
CREATE TABLE `yaf_sender_log` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `log_type` tinyint NOT NULL COMMENT '日志类型,如发送,取消等',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `sender_message_id` bigint unsigned NOT NULL COMMENT 'ID',
    `executor_type` varchar(32) NOT NULL DEFAULT '' COMMENT '执行发送类型',
    `message` varchar(255) NOT NULL COMMENT '发送相关消息',
    `status` tinyint NOT NULL COMMENT '操作状态',
    `create_time` bigint unsigned NOT NULL COMMENT '创建时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE,
    KEY `message_id` (`sender_message_id`) USING BTREE,
    KEY `sender_type` (`sender_type`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '后台发送日志';
CREATE TABLE `yaf_sender_tpl_config` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `name` varchar(32) NOT NULL COMMENT '名称',
    `tpl_id` varchar(32) NOT NULL COMMENT '模板KEY',
    `setting_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '配置ID',
    `config_data` text NOT NULL COMMENT '配置JSON数据',
    `status` tinyint NOT NULL COMMENT '状态',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更改用户id',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送模板配置';
CREATE TABLE `yaf_sender_tpl_body` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `tpl_id` varchar(32) NOT NULL COMMENT ' 模板ID',
    `tpl_data` text NOT NULL COMMENT '模板',
    `status` tinyint NOT NULL COMMENT '操作状态',
    `user_id` bigint unsigned NOT NULL COMMENT '操作用户id',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更改用户id',
    PRIMARY KEY (`id`),
    KEY `tpl_id` (`tpl_id`, `status`) USING BTREE,
    KEY `sender_type` (`sender_type`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送模板内容,有些接口不用这个';
CREATE TABLE `yaf_sender_mail_body` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID,由应用生成',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `tpl_id` varchar(32) NOT NULL COMMENT '模板ID',
    `tpl_var` varchar(512) NOT NULL DEFAULT '' COMMENT '模板变量',
    `max_try_num` smallint unsigned NOT NULL DEFAULT 1 COMMENT '最大发送次数',
    `status` tinyint NOT NULL COMMENT '启用状态:未完成,完成发送',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `expected_time` bigint unsigned NOT NULL COMMENT '预计发送时间',
    `finish_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '全部完成时间',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '用户id',
    `reply_mail` varchar(254) NOT NULL COMMENT '回复',
    `user_ip` varchar(40) NOT NULL DEFAULT '' COMMENT '操作者IP',
    `request_id` varchar(32) NOT NULL COMMENT '请求ID',
    `reply_host` varchar(255) NOT NULL COMMENT '结果返回主机',
    PRIMARY KEY (`id`),
    KEY `sender_record_data_IDX` (`expected_time`, `status`, `id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送邮件数据';
CREATE TABLE `yaf_sender_mail_message` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID,由应用生成',
    `snid` bigint unsigned NOT NULL COMMENT '消息ID',
    `sender_body_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '邮件内容ID',
    `to_mail` varchar(254) NOT NULL COMMENT '邮箱',
    `try_num` smallint unsigned NOT NULL DEFAULT 0 COMMENT '发送次数',
    `status` tinyint NOT NULL COMMENT '启用状态',
    `add_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '添加时间冗余',
    `send_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '发送时间',
    `receive_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '接收时间',
    `setting_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '发送使用配置ID',
    `res_data` varchar(512) NOT NULL COMMENT '返回数据',
    PRIMARY KEY (`id`),
    UNIQUE KEY(`snid`),
    KEY `sender_record_data_IDX` (`sender_body_id`) USING BTREE,
    KEY `sender_add_time_IDX` (`add_time`) USING BTREE,
    KEY `sender_res_data_IDX` (`setting_id`, `res_data`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送邮件收件人信息';
CREATE TABLE `yaf_sender_sms_body` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID,由应用生成',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `tpl_id` varchar(32) NOT NULL COMMENT '模板ID',
    `tpl_var` varchar(512) NOT NULL DEFAULT '' COMMENT '模板变量',
    `max_try_num` smallint unsigned NOT NULL DEFAULT 1 COMMENT '最大发送次数',
    `status` tinyint NOT NULL COMMENT '启用状态:未完成,完成发送',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `expected_time` bigint unsigned NOT NULL COMMENT '预计发送时间',
    `finish_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '全部完成时间',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '用户id',
    `user_ip` varchar(40) NOT NULL DEFAULT '' COMMENT '操作者IP',
    `request_id` varchar(32) NOT NULL COMMENT '请求ID',
    `reply_host` varchar(255) NOT NULL COMMENT '结果返回主机',
    PRIMARY KEY (`id`),
    KEY `sender_record_data_IDX` (`expected_time`, `status`, `id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送短信数据';
CREATE TABLE `yaf_sender_sms_message` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID,由应用生成',
    `snid` bigint unsigned NOT NULL COMMENT '消息ID',
    `sender_body_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '内容ID',
    `area` varchar(32) NOT NULL COMMENT '区号',
    `mobile` varchar(32) NOT NULL COMMENT '手机号',
    `try_num` smallint unsigned NOT NULL DEFAULT 0 COMMENT '发送次数',
    `status` tinyint NOT NULL COMMENT '启用状态',
    `add_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '添加时间冗余',
    `send_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '发送时间',
    `receive_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '接收时间',
    `setting_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '发送使用配置ID',
    `res_data` varchar(512) NOT NULL COMMENT '返回数据',
    PRIMARY KEY (`id`),
    UNIQUE KEY(`snid`),
    KEY `sender_record_data_IDX` (`sender_body_id`) USING BTREE,
    KEY `sender_add_time_IDX` (`add_time`) USING BTREE,
    KEY `sender_res_data_IDX` (`setting_id`, `res_data`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送短信接收手机号';
-- ----------- lsys-app-sender  ---------------