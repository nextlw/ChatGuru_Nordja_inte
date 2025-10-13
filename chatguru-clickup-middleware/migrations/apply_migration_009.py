#!/usr/bin/env python3
"""
Aplicador da Migration 009: Correção FINAL da lógica de mapeamento
Data: 2025-10-12

LÓGICA FINAL CORRETA:
- responsavel_nome = Nome do atendente (determina o Space)
- Info_1 = Empresa cliente (apenas campo personalizado)  
- Info_2 = Nome do cliente pessoa (determina a Folder)

Este script aplica a Migration 009 que corrige definitivamente o mapeamento
invertido que foi criado na Migration 008.
"""

import os
import sys
import logging
from pathlib import Path
from google.cloud.sql.connector import Connector
import sqlalchemy
from sqlalchemy import text
import pg8000

# Configurar logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

def get_connection():
    """Conecta ao Cloud SQL usando IAM authentication"""
    try:
        # Configurações do Cloud SQL
        project_id = "buzzlightear"
        region = "southamerica-east1"
        instance_name = "chatguru-middleware-db"
        database_name = "chatguru_middleware"
        
        # Inicializar o conector
        connector = Connector()
        
        def getconn():
            conn = connector.connect(
                f"{project_id}:{region}:{instance_name}",
                "pg8000",
                user="postgres",
                db=database_name,
                enable_iam_auth=True,
            )
            return conn
        
        # Criar engine
        engine = sqlalchemy.create_engine(
            "postgresql+pg8000://",
            creator=getconn,
        )
        
        logger.info("✅ Conexão com Cloud SQL estabelecida")
        return engine, connector
        
    except Exception as e:
        logger.error(f"❌ Erro ao conectar com Cloud SQL: {e}")
        raise

def apply_migration_009(engine):
    """Aplica a Migration 009"""
    try:
        # Ler arquivo da migração
        migration_file = Path(__file__).parent / "009_correct_mapping_logic.sql"
        
        if not migration_file.exists():
            raise FileNotFoundError(f"Arquivo de migração não encontrado: {migration_file}")
        
        with open(migration_file, 'r', encoding='utf-8') as f:
            migration_sql = f.read()
        
        logger.info("📂 Arquivo de migração 009 carregado")
        
        # Executar migração
        with engine.connect() as conn:
            # Executar em uma transação
            with conn.begin():
                logger.info("🚀 Iniciando aplicação da Migration 009...")
                
                # Dividir SQL em comandos individuais
                commands = [cmd.strip() for cmd in migration_sql.split(';') if cmd.strip()]
                
                for i, command in enumerate(commands, 1):
                    if command.strip():
                        try:
                            logger.info(f"🔄 Executando comando {i}/{len(commands)}")
                            result = conn.execute(text(command))
                            
                            # Se for um SELECT, mostrar resultado
                            if command.strip().upper().startswith('SELECT'):
                                rows = result.fetchall()
                                for row in rows:
                                    logger.info(f"📊 {row}")
                                    
                        except Exception as e:
                            logger.error(f"❌ Erro no comando {i}: {e}")
                            logger.error(f"💀 Comando que falhou: {command[:200]}...")
                            raise
                
                logger.info("✅ Migration 009 aplicada com sucesso!")
                
                # Verificar dados inseridos
                logger.info("🔍 Verificando dados inseridos...")
                
                # Contar registros por atendente
                count_result = conn.execute(text("""
                    SELECT attendant_name, COUNT(*) as total_clientes
                    FROM folder_mapping 
                    WHERE is_active = true 
                    GROUP BY attendant_name 
                    ORDER BY attendant_name
                """))
                
                logger.info("📊 Resumo dos mapeamentos por atendente:")
                for row in count_result:
                    logger.info(f"   {row[0]}: {row[1]} clientes")
                
                # Verificar aliases
                alias_result = conn.execute(text("""
                    SELECT attendant_alias, attendant_full_name, space_id
                    FROM attendant_aliases 
                    WHERE is_active = true 
                    ORDER BY attendant_alias
                """))
                
                logger.info("🔗 Aliases configurados:")
                for row in alias_result:
                    logger.info(f"   {row[0]} → {row[1]} (Space: {row[2]})")
                
        logger.info("🎉 Migration 009 aplicada e verificada com sucesso!")
        
    except Exception as e:
        logger.error(f"❌ Erro ao aplicar Migration 009: {e}")
        raise

def main():
    """Função principal"""
    try:
        logger.info("🚀 Iniciando aplicação da Migration 009...")
        logger.info("📋 Correção FINAL: responsavel_nome→Space, Info_2→Folder, Info_1→Campo")
        
        # Conectar ao banco
        engine, connector = get_connection()
        
        try:
            # Aplicar migração
            apply_migration_009(engine)
            
        finally:
            # Fechar conexões
            engine.dispose()
            connector.close()
            logger.info("🔌 Conexões fechadas")
            
        logger.info("✅ Processo concluído com sucesso!")
        
    except Exception as e:
        logger.error(f"💥 Erro fatal: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()