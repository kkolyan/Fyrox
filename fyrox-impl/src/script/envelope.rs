use std::any::Any;
use std::fmt::{Debug, Formatter};
use crate::engine::MessageTypeId;
use crate::script::{DynamicallyTypedScriptMessagePayload, ScriptMessagePayload};

/// the reason for this trait to exist is to allow dynamic type based message dispatching without
/// increasing memory footprint of statically typed messages.
/// with the cost of compilation time, of course.
pub(crate) trait ScriptMessagePayloadEnvelope: Any + Send + Debug {
    /// Returns `self` as `&dyn Any`
    fn as_any_ref(&self) -> &dyn Any;

    /// Returns `self` as `&dyn Any`
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_payload_mut(&mut self) -> &mut dyn ScriptMessagePayload;

    fn get_message_type_id(&self) -> MessageTypeId;
}

pub(crate) struct StaticallyTypedPayloadEnvelope<T: ScriptMessagePayload> {
    pub payload: T,
}

pub(crate) struct DynamicallyTypedPayloadEnvelope<T: DynamicallyTypedScriptMessagePayload> {
    pub payload: T,
}

impl<T: ScriptMessagePayload> Debug for StaticallyTypedPayloadEnvelope<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.payload.fmt(f)
    }
}

impl <T: ScriptMessagePayload> ScriptMessagePayloadEnvelope for StaticallyTypedPayloadEnvelope<T> {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn as_payload_mut(&mut self) -> &mut dyn ScriptMessagePayload {
        &mut self.payload
    }
    
    fn get_message_type_id(&self) -> MessageTypeId {
        MessageTypeId::Static(self.payload.type_id())
    }
}

impl <T: DynamicallyTypedScriptMessagePayload> Debug for DynamicallyTypedPayloadEnvelope<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.payload.fmt(f)
    }
}

impl <T: DynamicallyTypedScriptMessagePayload> ScriptMessagePayloadEnvelope for DynamicallyTypedPayloadEnvelope<T> {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn as_payload_mut(&mut self) -> &mut dyn ScriptMessagePayload {
        &mut self.payload
    }
    
    fn get_message_type_id(&self) -> MessageTypeId {
        MessageTypeId::Dynamic(self.payload.get_dynamic_type_id())
    }
}