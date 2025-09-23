# ChatGuru API Documentation

## Overview

ChatGuru is a WhatsApp automation platform that provides a comprehensive API for managing chats, messages, and automation workflows. This documentation covers the essential aspects of integrating with ChatGuru's API.

## Base Configuration

### API Endpoint
```
https://app.zap.guru/api/v1
```

### Authentication

All API requests require the following mandatory parameters:

| Parameter | Type | Description |
|-----------|------|-------------|
| `chat_number` | string | Full WhatsApp number with country code |
| `key` | string | API key (obtained from ChatGuru dashboard) |
| `account_id` | string | Your ChatGuru account ID |
| `phone_id` | string | Phone ID from ChatGuru |

### Access Requirements
- API feature must be activated in your ChatGuru account
- Only users with ADMIN permissions can access API settings
- API credentials are available in the "Celulares" (Phones) page

## Available Endpoints

### 1. Send Message
**Endpoint:** `POST /message_send`

Send a message to an existing WhatsApp chat.

**Parameters:**
- `chat_number` (required): Recipient's WhatsApp number
- `message` (required): Message content
- `key` (required): API key
- `account_id` (required): Account ID
- `phone_id` (required): Phone ID

**Response:**
```json
{
  "status": 200,
  "message": "Message sent successfully",
  "message_id": "MSG_123456"
}
```

### 2. Check Message Status
**Endpoint:** `GET /message_status`

Check the delivery status of a sent message.

**Parameters:**
- `message_id` (required): ID of the message to check
- `key` (required): API key
- `account_id` (required): Account ID
- `phone_id` (required): Phone ID

**Response:**
```json
{
  "status": 200,
  "delivery_status": "delivered",
  "read_status": "read",
  "timestamp": "2024-01-20T10:00:00Z"
}
```

### 3. Add New Chat
**Endpoint:** `POST /chat_add`

Add a new chat (requires "Add Chats" feature to be active).

**Parameters:**
- `chat_number` (required): New contact's WhatsApp number
- `name` (optional): Contact name
- `tags` (optional): Tags for categorization
- `key` (required): API key
- `account_id` (required): Account ID
- `phone_id` (required): Phone ID

**Response:**
```json
{
  "status": 200,
  "message": "Chat added successfully",
  "chat_id": "CHAT_789456"
}
```

### 4. Update Custom Fields
**Endpoint:** `PUT /chat_update_custom_fields`

Update custom fields for a specific chat.

**Parameters:**
- `chat_number` (required): WhatsApp number
- `custom_fields` (required): JSON object with field values
- `key` (required): API key
- `account_id` (required): Account ID
- `phone_id` (required): Phone ID

**Example Request:**
```json
{
  "chat_number": "5511999999999",
  "custom_fields": {
    "cliente_tipo": "Premium",
    "valor_compra": "1500.00",
    "data_ultimo_contato": "2024-01-20"
  },
  "key": "your_api_key",
  "account_id": "your_account_id",
  "phone_id": "your_phone_id"
}
```

### 5. Add Note to Chat
**Endpoint:** `POST /note_add`

Add an internal note to a chat conversation.

**Parameters:**
- `chat_number` (required): WhatsApp number
- `note` (required): Note content
- `key` (required): API key
- `account_id` (required): Account ID
- `phone_id` (required): Phone ID

### 6. Execute Dialog
**Endpoint:** `POST /dialog_execute`

Trigger a specific chatbot dialog/flow.

**Parameters:**
- `chat_number` (required): WhatsApp number
- `dialog_id` (required): ID of the dialog to execute
- `variables` (optional): Variables to pass to the dialog
- `key` (required): API key
- `account_id` (required): Account ID
- `phone_id` (required): Phone ID

### 7. Send File
**Endpoint:** `POST /message_file_send`

Send a file (image, document, audio, video) to a chat.

**Parameters:**
- `chat_number` (required): WhatsApp number
- `file_url` (required): Public URL of the file
- `caption` (optional): File caption
- `key` (required): API key
- `account_id` (required): Account ID
- `phone_id` (required): Phone ID

## Webhooks

ChatGuru supports webhooks for real-time event notifications.

### Webhook Events

1. **Message Received**
   - Triggered when a new message is received
   - Payload includes message content, sender info, timestamp

2. **Message Sent**
   - Triggered when a message is successfully sent
   - Includes message ID and delivery status

3. **Chat Status Changed**
   - Triggered when chat status changes (open, closed, waiting)
   - Includes previous and new status

4. **Custom Field Updated**
   - Triggered when custom fields are modified
   - Includes field names and values

### Webhook Configuration

Configure webhooks in your ChatGuru dashboard:
1. Navigate to Settings → Integrations
2. Add webhook URL
3. Select events to subscribe
4. Configure authentication (if required)

### Webhook Payload Example
```json
{
  "event_type": "message_received",
  "timestamp": "2024-01-20T10:00:00Z",
  "data": {
    "chat_number": "5511999999999",
    "message": "Hello, I need help",
    "message_type": "text",
    "sender_name": "John Doe",
    "chat_id": "CHAT_123456"
  },
  "account_id": "your_account_id",
  "phone_id": "your_phone_id"
}
```

## Integration with ClickUp

For the ChatGuru-ClickUp middleware integration, the following event mappings are used:

| ChatGuru Event | ClickUp Action |
|----------------|----------------|
| New chat created | Create new task |
| Message received with keyword | Update task description |
| Chat closed | Mark task as complete |
| Custom field updated | Update task custom fields |

## Error Handling

### Common Error Codes

| Code | Description | Solution |
|------|-------------|----------|
| 400 | Bad Request | Check parameter format |
| 401 | Unauthorized | Verify API key |
| 404 | Resource Not Found | Check chat_number or ID |
| 429 | Rate Limit Exceeded | Implement throttling |
| 500 | Internal Server Error | Contact support |

### Error Response Format
```json
{
  "status": 400,
  "error": "INVALID_PARAMETER",
  "message": "chat_number is required",
  "details": {
    "missing_fields": ["chat_number"]
  }
}
```

## Rate Limits

- **Default:** 60 requests per minute
- **Bulk operations:** 10 requests per minute
- **File uploads:** 30 per hour

## Best Practices

1. **Authentication Security**
   - Store API keys securely
   - Never expose keys in client-side code
   - Rotate keys periodically

2. **Error Handling**
   - Implement retry logic with exponential backoff
   - Log all API responses for debugging
   - Handle rate limits gracefully

3. **Data Management**
   - Cache frequently accessed data
   - Batch operations when possible
   - Use webhooks for real-time updates

4. **Testing**
   - Use sandbox environment for development
   - Test edge cases and error scenarios
   - Monitor API usage and performance

## SDK and Libraries

### Node.js Example
```javascript
const axios = require('axios');

class ChatGuruAPI {
  constructor(config) {
    this.baseURL = 'https://app.zap.guru/api/v1';
    this.key = config.key;
    this.accountId = config.accountId;
    this.phoneId = config.phoneId;
  }

  async sendMessage(chatNumber, message) {
    try {
      const response = await axios.post(`${this.baseURL}/message_send`, {
        chat_number: chatNumber,
        message: message,
        key: this.key,
        account_id: this.accountId,
        phone_id: this.phoneId
      });
      return response.data;
    } catch (error) {
      console.error('Error sending message:', error);
      throw error;
    }
  }
}
```

### Python Example
```python
import requests

class ChatGuruAPI:
    def __init__(self, key, account_id, phone_id):
        self.base_url = 'https://app.zap.guru/api/v1'
        self.key = key
        self.account_id = account_id
        self.phone_id = phone_id
    
    def send_message(self, chat_number, message):
        payload = {
            'chat_number': chat_number,
            'message': message,
            'key': self.key,
            'account_id': self.account_id,
            'phone_id': self.phone_id
        }
        
        response = requests.post(
            f'{self.base_url}/message_send',
            json=payload
        )
        return response.json()
```

## Support and Resources

- **Documentation:** https://oldwiki.chatguru.com.br/api/api-documentacao-v1
- **Developer Portal:** https://gurai.net/developer
- **Support Email:** support@chatguru.com.br
- **Community Forum:** https://community.chatguru.com.br

## Version History

- **v1.0** - Initial API release
- **v1.1** - Added webhook support
- **v1.2** - Custom fields API
- **v1.3** - File sending capabilities
- **v1.4** - Dialog execution endpoint

---

*Last updated: January 2024*