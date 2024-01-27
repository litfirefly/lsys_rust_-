rbac-access-check-res-empty = Resource [{$res}] operation [{$op}] for user [{$user_id}] not found, accessing user id:{$view_user_id}
rbac-access-check-access = User [{$user_id}]'s resource [{$res}:{$res_id}] operation [{$res_op}] is not authorized for you [{$view_user_id}] access
check-length = field [{$key}]check failed:{$msg}
parse-res-str-fail = Failed to parse permission string:{$token}
rbac-res-exits = resource [{$name}:{$key}] already exists
rbac-priority-range = Role priority needs to be between [{$min}-{$max}].
rbac-role-exist = role [{$name}] already exists
rbac-relation-key-exist = Role KEY {$relation_key} is already used by role [{$name}:{$id}].
rbac-miss-relation-key = Role KEY{$relation_key} cannot be null
rbac-res-op-user-wrong = This role [{$name}:{$role_id}] cannot be associated with user [{$range}].
rbac-res-op-range-wrong = This role [{$name}:{$role_id}] cannot be associated with a resource [{$range}].
rbac-role-miss-res = role [{$name}:{$id}] does not exist
rbac-role-miss-res-op = Undiscovered resource [{$name}:{$id}]
rbac-role-bad-res-user = Non-system roles cannot add user resources that are not part of this role, resource not :{$res}, user is :{$user_id}
rbac-role-wrong-res-op = Found that the {$res_id} of [{$name}:{$id}] in the system does not match the {$p_res_id} passed in.
rbac-user-range-bad = Associated roles should be added using a specialized interface.