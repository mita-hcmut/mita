#! /bin/sh

set -e
set -u
set -o pipefail

vault_addr="vault:8200"
oauth2_addr="oauth2:8443"

until wget -q --spider http://$vault_addr/v1/sys/health; do
    >&2 echo "waiting for vault container..."
    sleep 1
done

vault login root

cat << EOF | vault policy write kv-policy -
path "secret/data/{{identity.entity.id}}" {
  capabilities = ["create", "read", "update", "patch", "delete", "list"]
}
EOF

vault auth enable jwt

until wget -q --spider http://$oauth2_addr/isalive ; do
    >&2 echo "waiting for oauth container..."
    sleep 1
done

vault write auth/jwt/config oidc_discovery_url=http://$oauth2_addr/default

vault write auth/jwt/role/user bound_audiences=client_id user_claim=sub role_type=jwt policies=kv-policy

vault token revoke -self