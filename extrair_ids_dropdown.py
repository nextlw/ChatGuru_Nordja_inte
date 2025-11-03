import yaml

# Caminho para o arquivo YAML do prompt
yaml_path = "chatguru-clickup-middleware/config/ai_prompt.yaml"
dropdown_field_id = "0ed63eec-1c50-4190-91c1-59b4b17557f6"

def find_dropdown_options(data, field_id):
    if isinstance(data, dict):
        if data.get("id") == field_id and "type_config" in data:
            return data["type_config"].get("options", [])
        for v in data.values():
            found = find_dropdown_options(v, field_id)
            if found:
                return found
    elif isinstance(data, list):
        for item in data:
            found = find_dropdown_options(item, field_id)
            if found:
                return found
    return None

if __name__ == "__main__":
    with open(yaml_path, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)

    options = find_dropdown_options(data, dropdown_field_id)

    if not options:
        print("Nenhuma opção encontrada para o campo:", dropdown_field_id)
    else:
        print("IDs das opções do menu suspenso:")
        for opt in options:
            print(opt.get("id"), "-", opt.get("name"))