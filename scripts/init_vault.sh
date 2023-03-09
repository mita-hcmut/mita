#! /bin/sh

set -e
set -u
set -x
set -o pipefail

vault_addr="vault:8200"
oauth2_addr="oauth2:8443"

until wget -q --spider http://$vault_addr/v1/sys/health; do
    >&2 echo "waiting for vault container..."
    sleep 1
done

vault login root

user_data_path="user-data"

cat << EOF | vault policy write moodle-policy -
path "$user_data_path/data/+/{{identity.entity.id}}" {
  capabilities = ["create", "read", "update", "patch", "delete", "list"]
}
EOF

vault secrets enable -path=$user_data_path kv-v2 || true
vault auth enable -path=$user_data_path jwt || true

until wget -q --spider http://$oauth2_addr/isalive ; do
    >&2 echo "waiting for oauth container..."
    sleep 1
done

vault write auth/$user_data_path/config oidc_discovery_url=http://$oauth2_addr/default

vault write auth/$user_data_path/role/user bound_audiences=client_id user_claim=sub role_type=jwt policies=moodle-policy

# vault token revoke -self