//! Custom Fields do ClickUp
//!
//! A API do ClickUp suporta 18 tipos diferentes de custom fields, cada um com
//! formato de valor específico.
//!
//! ⚠️ IMPORTANTE: Checkbox fields usam string "true"/"false", NÃO boolean!
//! ⚠️ IMPORTANTE: Timestamps são em MILISSEGUNDOS, não segundos!

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Representa um custom field do ClickUp
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomField {
    /// ID do custom field (UUID)
    pub id: String,

    /// Nome do campo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tipo do campo (ver CustomFieldType)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    /// Configuração específica do tipo de campo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_config: Option<TypeConfig>,

    /// Valor do campo (formato depende do tipo)
    pub value: CustomFieldValue,
}

/// Configuração específica de cada tipo de campo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeConfig {
    /// Para dropdown/labels: opções disponíveis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<DropdownOption>>,

    /// Para campos numéricos: formato padrão
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<i32>,

    /// Placeholder text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Para date fields: include time?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_time: Option<bool>,
}

/// Opção de dropdown/labels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DropdownOption {
    /// ID da opção (UUID)
    pub id: String,

    /// Nome da opção
    pub name: String,

    /// Cor da opção (hex color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Ordem da opção
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderindex: Option<i32>,
}

/// Enum com todos os 18 tipos de valores de custom fields
///
/// ⚠️ CRÍTICO: Checkbox usa String("true"/"false"), NÃO bool!
/// ⚠️ CRÍTICO: Timestamps são i64 em MILISSEGUNDOS!
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomFieldValue {
    // ==================== TEXT TYPES ====================
    /// Text field - string simples
    Text(String),

    // ==================== NUMBER TYPES ====================
    /// Number field - número inteiro ou decimal (API aceita string ou number)
    Number(f64),

    /// Currency field - valor monetário em centavos (API usa number)
    Currency(i64),

    // ==================== BOOLEAN TYPE ====================
    /// ⚠️ CRÍTICO: Checkbox field - USA STRING "true"/"false", NÃO BOOLEAN!
    /// Exemplo: {"value": "true"} ou {"value": "false"}
    Checkbox(String),

    // ==================== SELECTION TYPES ====================
    /// Dropdown field - single select (option ID como string)
    Dropdown(String),

    /// Labels field - multiple select (array de option IDs)
    Labels(Vec<String>),

    // ==================== DATE/TIME TYPES ====================
    /// Date field - timestamp em MILISSEGUNDOS (i64)
    /// Exemplo: 1672531200000 para 2023-01-01 00:00:00 UTC
    Date(i64),

    // ==================== RELATIONSHIP TYPES ====================
    /// Users field - array de user IDs (numbers)
    Users(Vec<u32>),

    /// Email field - endereço de email
    Email(String),

    /// Phone field - número de telefone
    Phone(String),

    /// URL field - URL válida
    Url(String),

    // ==================== LOCATION TYPE ====================
    /// Location field - objeto com location data
    Location(LocationValue),

    // ==================== RATING TYPE ====================
    /// Rating field - rating de 0 a 5 (integer)
    Rating(i32),

    // ==================== FILE TYPE ====================
    /// Attachment/Files field - array de file objects
    Files(Vec<FileValue>),

    // ==================== FORMULA/AUTO TYPE ====================
    /// Automatic field - calculado automaticamente (read-only)
    Automatic(JsonValue),

    // ==================== FALLBACK ====================
    /// Para tipos desconhecidos ou null values
    Null,
}

/// Valor de location field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationValue {
    /// Endereço formatado
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_address: Option<String>,

    /// Latitude
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,

    /// Longitude
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
}

/// Valor de file field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileValue {
    /// ID do arquivo
    pub id: String,

    /// Nome do arquivo
    pub name: String,

    /// URL do arquivo
    pub url: String,

    /// Tamanho em bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,
}

impl CustomField {
    /// Cria um custom field text
    pub fn text(id: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("text".to_string()),
            type_config: None,
            value: CustomFieldValue::Text(value.into()),
        }
    }

    /// Cria um custom field number
    pub fn number(id: impl Into<String>, value: f64) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("number".to_string()),
            type_config: None,
            value: CustomFieldValue::Number(value),
        }
    }

    /// ⚠️ Cria um custom field checkbox (usa string "true"/"false")
    pub fn checkbox(id: impl Into<String>, checked: bool) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("checkbox".to_string()),
            type_config: None,
            value: CustomFieldValue::Checkbox(if checked {
                "true".to_string()
            } else {
                "false".to_string()
            }),
        }
    }

    /// Cria um custom field dropdown (single select)
    pub fn dropdown(id: impl Into<String>, option_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("drop_down".to_string()),
            type_config: None,
            value: CustomFieldValue::Dropdown(option_id.into()),
        }
    }

    /// Cria um custom field labels (multiple select)
    pub fn labels(id: impl Into<String>, option_ids: Vec<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("labels".to_string()),
            type_config: None,
            value: CustomFieldValue::Labels(option_ids),
        }
    }

    /// ⚠️ Cria um custom field date (timestamp em MILISSEGUNDOS)
    pub fn date(id: impl Into<String>, timestamp_ms: i64) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("date".to_string()),
            type_config: None,
            value: CustomFieldValue::Date(timestamp_ms),
        }
    }

    /// Cria um custom field users
    pub fn users(id: impl Into<String>, user_ids: Vec<u32>) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("users".to_string()),
            type_config: None,
            value: CustomFieldValue::Users(user_ids),
        }
    }

    /// Cria um custom field email
    pub fn email(id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("email".to_string()),
            type_config: None,
            value: CustomFieldValue::Email(email.into()),
        }
    }

    /// Cria um custom field URL
    pub fn url(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("url".to_string()),
            type_config: None,
            value: CustomFieldValue::Url(url.into()),
        }
    }

    /// Cria um custom field phone
    pub fn phone(id: impl Into<String>, phone: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("phone".to_string()),
            type_config: None,
            value: CustomFieldValue::Phone(phone.into()),
        }
    }

    /// Cria um custom field currency (em centavos)
    pub fn currency(id: impl Into<String>, cents: i64) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("currency".to_string()),
            type_config: None,
            value: CustomFieldValue::Currency(cents),
        }
    }

    /// Cria um custom field rating (0-5)
    pub fn rating(id: impl Into<String>, rating: i32) -> Self {
        Self {
            id: id.into(),
            name: None,
            type_: Some("rating".to_string()),
            type_config: None,
            value: CustomFieldValue::Rating(rating.clamp(0, 5)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_uses_string() {
        let field_true = CustomField::checkbox("test-id", true);
        match field_true.value {
            CustomFieldValue::Checkbox(ref s) => assert_eq!(s, "true"),
            _ => panic!("Expected Checkbox variant"),
        }

        let field_false = CustomField::checkbox("test-id", false);
        match field_false.value {
            CustomFieldValue::Checkbox(ref s) => assert_eq!(s, "false"),
            _ => panic!("Expected Checkbox variant"),
        }
    }

    #[test]
    fn test_date_uses_milliseconds() {
        // 2023-01-01 00:00:00 UTC = 1672531200000 milliseconds
        let field = CustomField::date("test-id", 1672531200000);
        match field.value {
            CustomFieldValue::Date(ms) => assert_eq!(ms, 1672531200000),
            _ => panic!("Expected Date variant"),
        }
    }

    #[test]
    fn test_rating_clamps() {
        let field = CustomField::rating("test-id", 10);
        match field.value {
            CustomFieldValue::Rating(r) => assert_eq!(r, 5), // Clamped to max
            _ => panic!("Expected Rating variant"),
        }
    }

    #[test]
    fn test_custom_field_constructors() {
        let text = CustomField::text("1", "hello");
        assert_eq!(text.id, "1");
        assert!(matches!(text.value, CustomFieldValue::Text(_)));

        let number = CustomField::number("2", 42.5);
        assert!(matches!(number.value, CustomFieldValue::Number(_)));

        let dropdown = CustomField::dropdown("3", "option-1");
        assert!(matches!(dropdown.value, CustomFieldValue::Dropdown(_)));

        let labels = CustomField::labels("4", vec!["opt1".to_string(), "opt2".to_string()]);
        assert!(matches!(labels.value, CustomFieldValue::Labels(_)));
    }
}
