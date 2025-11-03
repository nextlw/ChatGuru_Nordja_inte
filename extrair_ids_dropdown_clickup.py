import requests
import os

# Parâmetros necessários
CLICKUP_API_TOKEN = os.getenv("CLICKUP_API_TOKEN") or "COLOQUE_SUA_API_KEY_AQUI"
FIELD_ID = "0ed63eec-1c50-4190-91c1-59b4b17557f6"

# Você pode precisar do ID de uma lista (list_id) para buscar os campos customizados dela
# Exemplo: LIST_ID = "901320755706"
LIST_ID = "COLOQUE_O_LIST_ID_AQUI"

def get_custom_fields(list_id):
    url = f"https://api.clickup.com/api/v2/list/{list_id}/field"
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
    fields_data = get_custom_fields(LIST_ID)
    fields = fields_data.get("fields", [])
    options = extract_dropdown_options(fields, FIELD_ID)
    if not options:
        print("Nenhuma opção encontrada para o campo:", FIELD_ID)
    else:
        print("IDs das opções do menu suspenso:")
        for opt in options:
            print(opt.get("id"), "-", opt.get("name"))