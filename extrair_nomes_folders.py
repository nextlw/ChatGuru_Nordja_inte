import json
import sys

def extrair_nomes(path_entrada, path_saida):
    with open(path_entrada, "r", encoding="utf-8") as f:
        folders = json.load(f)
    nomes = sorted([folder["name"] for folder in folders], key=lambda x: x.lower())
    with open(path_saida, "w", encoding="utf-8") as f:
        json.dump(nomes, f, ensure_ascii=False, indent=2)

if __name__ == "__main__":
    # Uso: python3 extrair_nomes_folders.py folders_hie.json nomes_folders.json
    if len(sys.argv) < 3:
        print("Uso: python3 extrair_nomes_folders.py <entrada.json> <saida.json>")
        sys.exit(1)
    extrair_nomes(sys.argv[1], sys.argv[2])