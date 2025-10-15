## Política de Fallback para MCP

Para garantir maior resiliência no uso das ferramentas, foi criada uma política de fallback automático que, ao detectar falha pela segunda vez em uma ferramenta interna ou nativa, tenta utilizar a ferramenta MCP correspondente como alternativa.

Essa política está definida no arquivo de configuração YAML separado:

`.elai/rules/fallback_mcp_rules.yaml`

Este arquivo contém a regra que monitora falhas consecutivas e aciona o fallback para MCP, garantindo continuidade das operações.