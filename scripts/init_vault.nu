let login_command = "vault login root"
let enable_command = "vault auth enable jwt"
let config_command = "vault write auth/jwt/config oidc_discovery_url='http://oauth2:8443/default'"
let role_command = "vault write auth/jwt/role/user bound_audiences='client_id' user_claim='sub' role_type='jwt'"

docker exec mita-vault-1 sh -c $"($login_command) && ($enable_command) && ($config_command) && ($role_command)"