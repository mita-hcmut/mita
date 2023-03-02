let login_command = "vault login root"

let policy = 'path "secret/data/{{identity.entity.id}}" {
  capabilities = ["create", "read", "update", "patch", "delete", "list"]
}'

docker exec mita-vault-1 sh -c $"echo '($policy)' | vault policy write kv-policy -"

let enable_command = "vault auth enable jwt"
let config_command = "vault write auth/jwt/config oidc_discovery_url='http://oauth2:8443/default'"
let role_command = "vault write auth/jwt/role/user bound_audiences='client_id' user_claim='sub' role_type='jwt' policies='kv-policy'"

docker exec mita-vault-1 sh -c $"($login_command) && ($enable_command) && ($config_command) && ($role_command)"
