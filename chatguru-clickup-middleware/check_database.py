#!/usr/bin/env python3
"""
Script para verificar se o banco de dados está populado via API endpoint.
Não requer dependências extras - usa apenas curl via subprocess.
"""
import json
import subprocess
import sys

def run_command(cmd):
    """Execute command and return output."""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30)
        return result.stdout.strip(), result.returncode
    except subprocess.TimeoutExpired:
        return None, -1

def main():
    project_id = "buzzlightear"

    print("=" * 80)
    print("📊 VERIFICAÇÃO DO BANCO DE DADOS - ESTRUTURA CLICKUP")
    print("=" * 80)

    # Get Cloud Run URL
    print("\n🌐 Obtendo URL do Cloud Run...")
    url, code = run_command(
        f"gcloud run services describe chatguru-clickup-middleware "
        f"--region=southamerica-east1 "
        f"--format='value(status.url)' "
        f"--project={project_id} 2>/dev/null"
    )

    if code != 0 or not url:
        print("❌ Erro ao obter URL do Cloud Run")
        print("\nVerifique se o serviço está deployado:")
        print(f"  gcloud run services list --project={project_id} --region=southamerica-east1")
        sys.exit(1)

    print(f"✅ URL: {url}")

    # Fetch database check endpoint
    print(f"\n📡 Consultando {url}/admin/db-check...")
    response, code = run_command(f"curl -s '{url}/admin/db-check'")

    if code != 0 or not response:
        print("❌ Erro ao fazer requisição HTTP")
        sys.exit(1)

    try:
        data = json.loads(response)

        # Status geral
        status = data.get("status", "Unknown")
        print(f"\n{status}")
        print("=" * 80)

        # Summary
        summary = data.get("summary", {})
        print(f"\n📊 RESUMO:")
        print(f"  Spaces:  {summary.get('active_spaces', 0)}/{summary.get('total_spaces', 0)} ativos")
        print(f"  Folders: {summary.get('active_folders', 0)}/{summary.get('total_folders', 0)} ativos")
        print(f"  Lists:   {summary.get('active_lists', 0)}/{summary.get('total_lists', 0)} ativos")

        # Spaces
        spaces_data = data.get("spaces", {})
        spaces = spaces_data.get("data", [])
        missing = spaces_data.get("missing", [])

        print(f"\n🌐 SPACES ({len(spaces)}):")
        for space in spaces:
            status_icon = "✅" if space.get("is_active") else "❌"
            print(f"  {status_icon} {space.get('space_name', 'Unknown')}")

        if missing:
            print(f"\n⚠️  ESPAÇOS FALTANDO ({len(missing)}):")
            for m in missing:
                print(f"  ❌ {m}")

        # Folder mappings
        folder_data = data.get("folder_mappings", {})
        folders = folder_data.get("data", [])
        print(f"\n📁 FOLDER MAPPINGS ({len(folders)}):")
        for folder in folders[:15]:  # Mostrar apenas os primeiros 15
            status_icon = "✅" if folder.get("is_active") else "❌"
            client = folder.get("client_name", "?")
            attendant = folder.get("attendant_name", "?")
            path = folder.get("folder_path", "?")
            print(f"  {status_icon} {client:20s} + {attendant:20s} → {path}")

        if len(folders) > 15:
            print(f"  ... e mais {len(folders) - 15} folders")

        # Lists
        list_data = data.get("list_cache", {})
        lists = list_data.get("data", [])
        print(f"\n📋 LISTS (mostrando {min(len(lists), 15)}/{len(lists)}):")
        for lst in lists[:15]:
            status_icon = "✅" if lst.get("is_active") else "❌"
            name = lst.get("list_name", "?")[:40]
            folder_id = lst.get("folder_id", "?")[:15]
            print(f"  {status_icon} {name:40s} | Folder: {folder_id}")

        print("\n" + "=" * 80)
        print("✅ Verificação concluída!")
        print("=" * 80)

    except json.JSONDecodeError:
        print("❌ Resposta não é JSON válido:")
        print(response[:500])
        sys.exit(1)

if __name__ == "__main__":
    main()
