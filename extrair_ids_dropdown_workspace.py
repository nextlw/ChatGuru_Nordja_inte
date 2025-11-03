import requests
import os

# Parâmetros necessários
CLICKUP_API_TOKEN = os.getenv("CLICKUP_API_TOKEN") or "COLOQUE_SUA_API_KEY_AQUI"
TEAM_ID = "9013037641"  # Exemplo: "9013037641"
FIELD_ID = "0ed63eec-1c50-4190-91c1-59b4b17557f6"

def get_all_custom_fields(team_id):
    url = f"https://api.clickup.com/api/v2/team/{team_id}/field"
    headers = {
        "Authorization": CLICKUP_API_TOKEN
    }
    resp = requests.get(url, headers=headers)
    resp.raise_for_status()
    return resp.json()

def extract_dropdown_options(fields, field_id):
    for field in fields:
        if field.get("id") == field_id and field.get("type") == "drop_down":
            options = field.get("type_config", {}).get("options", [])
            return options
    return []

if __name__ == "__main__":
    fields_data = get_all_custom_fields(TEAM_ID)
    fields = fields_data.get("fields", [])
    options = extract_dropdown_options(fields, FIELD_ID)
    if not options:
        print("Nenhuma opção encontrada para o campo:", FIELD_ID)
    else:
        print("IDs das opções do menu suspenso:")
        for opt in options:
            print(opt.get("id"), "-", opt.get("name"))