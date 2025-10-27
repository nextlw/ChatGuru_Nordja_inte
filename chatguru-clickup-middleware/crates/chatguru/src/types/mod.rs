pub mod payload;
pub mod webhook;

// Re-export dos tipos principais para conveniência
pub use payload::{
    ChatGuruPayload,
    BotContext,
    EventTypePayload,
    EventData,
    GenericPayload,
};

pub use webhook::WebhookPayload;
