# ChatGuru API Client

Cliente Rust para a API do ChatGuru com suporte completo a webhooks e envio de mensagens.

## Funcionalidades

- ✅ **Cliente HTTP tipo-seguro** para a API ChatGuru
- ✅ **Adicionar anotações** aos chats
- ✅ **Enviar mensagens de confirmação** via WhatsApp
- ✅ **Tipos de webhook** flexíveis (ChatGuru, EventType, Generic)
- ✅ **Normalização de campos de mídia** (imagens, áudios, vídeos)
- ✅ **Timeouts configuráveis** (10s timeout, 3s connect timeout)

## Exemplo de Uso

```rust
use chatguru::{ChatGuruClient, types::ChatGuruPayload};

#[tokio::main]
async fn main() -> chatguru::Result<()> {
    // IMPORTANTE: Ler de variáveis de ambiente
    let api_token = std::env::var("CHATGURU_API_TOKEN")
        .expect("CHATGURU_API_TOKEN não configurado");
    let api_endpoint = "https://api.chatguru.app/api/v1";
    let account_id = std::env::var("CHATGURU_ACCOUNT_ID")
        .expect("CHATGURU_ACCOUNT_ID não configurado");

    let client = ChatGuruClient::new(api_token, api_endpoint, account_id);

    // Adicionar anotação ao chat
    client.add_annotation(
        "chat_123",
        "5511999999999",
        "Tarefa criada no ClickUp: TASK-123"
    ).await?;

    // Enviar mensagem de confirmação
    client.send_confirmation_message(
        "5511999999999",
        None,
        "✅ Sua solicitação foi registrada!"
    ).await?;

    Ok(())
}
```

## Tipos de Webhook

O crate suporta múltiplos formatos de webhook através do enum `WebhookPayload`:

- **ChatGuru**: Formato atual do ChatGuru com campos personalizados
- **EventType**: Formato legado com event_type
- **Generic**: Formato genérico/mínimo

## API do ChatGuru

Este crate implementa os seguintes endpoints:

### Adicionar Anotação
```
POST {api_endpoint}?key={token}&account_id={id}&phone_id={phone_id}&action=note_add&note_text={text}&chat_number={number}
```

### Enviar Mensagem
```
POST {api_endpoint}?key={token}&account_id={id}&phone_id={phone_id}&action=message_send&text={text}&chat_number={number}
```

## Configuração

O cliente é configurado através de variáveis de ambiente:

- `CHATGURU_API_TOKEN`: Token de autenticação da API
- `CHATGURU_API_ENDPOINT`: URL base da API (padrão: `https://api.chatguru.app/api/v1`)
- `CHATGURU_ACCOUNT_ID`: ID da conta ChatGuru

## Tratamento de Erros

Todos os métodos retornam `chatguru::Result<T>`, que é um alias para `Result<T, ChatGuruError>`.

Os erros são categorizados em:
- **NetworkError**: Falhas de rede/HTTP
- **ApiError**: Erros retornados pela API
- **SerializationError**: Erros de serialização/deserialização JSON
- **ValidationError**: Dados inválidos
- **InternalError**: Erros internos do cliente

## Licença

Propriedade de eLai Integration Team
