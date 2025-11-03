# USO AUTOMÁTICO:
# Para rodar automaticamente e gerar o JSON de todos os folders a partir de um arquivo de texto:
# Exemplo: python3 extrair_folders_clickup.py clickup_hierarchy.txt > folders.json
#
# Se quiser extrair direto de um arquivo já existente no projeto (ex: clickup_hierarchy.json ou .txt):
# python3 extrair_folders_clickup.py clickup_hierarchy.json > folders.json
#
# O script irá ler o arquivo, extrair todos os folders e IDs e gerar o JSON na saída padrão.
import re
import json
import sys

def extrair_folders(texto):
    # Regex para pegar nome do folder e o ID
    # Regex aprimorado: captura o nome completo do folder (incluindo parênteses internos) sem pegar o "(Folder ID: ...)"
    padrao = re.compile(
        r"^[\s│╎┃]*[├└]──\s*(.+?)\s*\(Folder ID:\s*(\d+)\)", re.MULTILINE
    )
    folders = []
    for match in padrao.finditer(texto):
        nome = match.group(1).rstrip()
        folder_id = match.group(2).strip()
        folders.append({"name": nome, "id": folder_id})
    return folders

if __name__ == "__main__":
    # Pode receber o texto de um arquivo ou stdin
    if len(sys.argv) > 1:
        with open(sys.argv[1], "r", encoding="utf-8") as f:
            texto = f.read()
    else:
        print("Cole o texto e pressione Ctrl+D para finalizar:")
        texto = sys.stdin.read()

    resultado = extrair_folders(texto)
    print(json.dumps(resultado, ensure_ascii=False, indent=2))