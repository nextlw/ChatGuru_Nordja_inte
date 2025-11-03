import requests
import sys
import toml

# CONFIG
CONFIG_PATH = "chatguru-clickup-middleware/config/default.toml"

def get_clickup_token_from_toml(config_path=CONFIG_PATH):
    config = toml.load(config_path)
    token = config.get("clickup", {}).get("token")
    if not token:
        token = config.get("CLICKUP_AUTH_TOKEN")
    if not token:
        raise Exception("Token não encontrado no arquivo TOML.")
    return token

def listar_campos_personalizados(list_id, token):
    url = f"https://api.clickup.com/api/v2/list/{list_id}/field"
    headers = {
        "Authorization": token,
        "Content-Type": "application/json"
    }
    resp = requests.get(url, headers=headers)
    if resp.status_code != 200:
        print(f"Erro ao buscar campos personalizados: {resp.status_code} - {resp.text}")
        sys.exit(1)
    data = resp.json()
    return data

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Lista todos os campos personalizados de uma lista ClickUp.")
    parser.add_argument("--list", required=True, help="ID da lista ClickUp para buscar campos personalizados")
    parser.add_argument("--config", default=CONFIG_PATH, help="Caminho do arquivo TOML com o token OAuth2")
    args = parser.parse_args()

    token = get_clickup_token_from_toml(args.config)
    campos = listar_campos_personalizados(args.list, token)
    print("Campos personalizados encontrados na lista:")
    print()
    for campo in campos.get("fields", []):
        print(f"Nome: {campo.get('name')}")
        print(f"ID: {campo.get('id')}")
        print(f"Tipo: {campo.get('type')}")
        print(f"Opções: {campo.get('type_config', {}).get('options', [])}")
        print("-" * 40)