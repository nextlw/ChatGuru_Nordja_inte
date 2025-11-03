import requests
import json
import sys
import toml

# Caminho do config padrão do projeto
CONFIG_PATH = "chatguru-clickup-middleware/config/default.toml"

def get_clickup_token_from_toml(config_path=CONFIG_PATH):
    try:
        config = toml.load(config_path)
        # Preferência para CLICKUP_AUTH_TOKEN, depois token do bloco [clickup]
        token = config.get("clickup", {}).get("token")
        if not token:
            token = config.get("CLICKUP_AUTH_TOKEN")
        if not token:
            raise Exception("Token não encontrado no arquivo TOML.")
        return token
    except Exception as e:
        print(f"Erro ao ler token do arquivo {config_path}: {e}")
        sys.exit(1)

def get_customfield_options(custom_field_id, headers):
    url = f"https://api.clickup.com/api/v2/custom_field/{custom_field_id}"
    resp = requests.get(url, headers=headers)
    if resp.status_code == 404:
        print(f"ERRO 404: Campo personalizado '{custom_field_id}' não encontrado ou sem permissão de acesso.\nVerifique o ID, o token e o workspace.")
        sys.exit(1)
    resp.raise_for_status()
    data = resp.json()
    return data.get("type_config", {}).get("options", [])

def delete_option(custom_field_id, option_id, headers):
    url = f"https://api.clickup.com/api/v2/custom_field/{custom_field_id}/option/{option_id}"
    resp = requests.delete(url, headers=headers)
    if resp.status_code not in (200,204):
        print(f"Erro ao excluir opção {option_id}: {resp.text}")

def add_option(custom_field_id, name, headers):
    url = f"https://api.clickup.com/api/v2/custom_field/{custom_field_id}/option"
    payload = {"name": name}
    resp = requests.post(url, headers=headers, data=json.dumps(payload))
    if resp.status_code not in (200,201):
        print(f"Erro ao adicionar opção '{name}': {resp.text}")

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Atualiza opções de um campo personalizado do ClickUp com nomes de folders.")
    parser.add_argument("--field", required=True, help="ID do campo personalizado do ClickUp")
    parser.add_argument("--json", default="folders_hie.json", help="Arquivo JSON com nomes dos folders")
    parser.add_argument("--config", default=CONFIG_PATH, help="Caminho do arquivo TOML com o token OAuth2")
    args = parser.parse_args()

    token = get_clickup_token_from_toml(args.config)
    custom_field_id = args.field
    headers = {
        "Authorization": token,
        "Content-Type": "application/json"
    }

    # 1. Carrega nomes dos folders do arquivo JSON
    with open(args.json, "r", encoding="utf-8") as f:
        folders = json.load(f)
    nomes = [f["name"] for f in folders]

    # 2. Busca e exclui todas as opções existentes
    print("Buscando opções atuais do campo personalizado...")
    opcoes = get_customfield_options(custom_field_id, headers)
    for opt in opcoes:
        print(f"Excluindo: {opt['name']} ({opt['id']})")
        delete_option(custom_field_id, opt["id"], headers)

    # 3. Adiciona uma opção para cada nome de folder
    print("Adicionando opções...")
    for nome in nomes:
        print(f"Adicionando: {nome}")
        add_option(custom_field_id, nome, headers)

    print("Processo concluído.")

if __name__ == "__main__":
    main()