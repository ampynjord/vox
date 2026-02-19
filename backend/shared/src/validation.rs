use validator::Validate;
use crate::errors::{VoxError, VoxResult};

/// Valide une requête avec les règles de validation définies
pub fn validate_request<T: Validate>(req: &T) -> VoxResult<()> {
    req.validate().map_err(|e| {
        let messages: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |err| {
                    format!(
                        "{}: {}",
                        field,
                        err.message.as_deref().unwrap_or("invalide")
                    )
                })
            })
            .collect();
        VoxError::ValidationError(messages.join(", "))
    })
}
