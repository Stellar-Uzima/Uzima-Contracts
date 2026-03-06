use soroban_sdk::{contracttype, Address, Env, Map, String, Vec};
use crate::{types::*, errors::Error, events::*};

pub struct ProfessionalDirectoryManager;

impl ProfessionalDirectoryManager {
    pub fn register_professional(
        env: &Env,
        professional_id: Address,
        name: String,
        credentials: Vec<Credential>,
        specializations: Vec<String>,
        languages: Vec<String>,
        availability: AvailabilitySchedule,
        contact_info: ContactInfo,
        bio: String,
        insurance_accepted: Vec<String>,
    ) -> Result<(), Error> {
        let professional = MentalHealthProfessional {
            professional_id: professional_id.clone(),
            name,
            credentials,
            specializations,
            languages,
            availability,
            contact_info,
            bio,
            rating: 0.0,
            review_count: 0,
            verified: false,
            insurance_accepted,
        };

        env.storage().instance().set(&professional_id, &professional);

        env.events().publish(
            (Symbol::new(env, "professional_registered"),),
            professional_id,
        );

        Ok(())
    }

    pub fn update_availability(
        env: &Env,
        professional_id: Address,
        availability: AvailabilitySchedule,
    ) -> Result<(), Error> {
        let mut professional: MentalHealthProfessional = env.storage().instance()
            .get(&professional_id)
            .ok_or(Error::ProfessionalNotFound)?;

        professional.availability = availability;
        env.storage().instance().set(&professional_id, &professional);

        Ok(())
    }

    pub fn search_professionals(
        env: &Env,
        specialization: Option<String>,
        language: Option<String>,
        insurance: Option<String>,
        max_results: u32,
    ) -> Vec<MentalHealthProfessional> {
        // In a real implementation, this would query an index
        // For now, return empty vec as we can't enumerate all professionals
        Vec::new(env)
    }

    pub fn get_professional_profile(
        env: &Env,
        professional_id: Address,
    ) -> Result<MentalHealthProfessional, Error> {
        env.storage().instance()
            .get(&professional_id)
            .ok_or(Error::ProfessionalNotFound)
    }

    pub fn submit_review(
        env: &Env,
        professional_id: Address,
        reviewer_id: Address,
        rating: u32,
        review_text: String,
    ) -> Result<(), Error> {
        if rating > 5 {
            return Err(Error::InvalidInput);
        }

        let mut professional: MentalHealthProfessional = env.storage().instance()
            .get(&professional_id)
            .ok_or(Error::ProfessionalNotFound)?;

        // Update rating (simple average)
        let total_rating = professional.rating * professional.review_count as f32 + rating as f32;
        professional.review_count += 1;
        professional.rating = total_rating / professional.review_count as f32;

        env.storage().instance().set(&professional_id, &professional);

        // Store review (simplified)
        let review = ProfessionalReview {
            reviewer_id,
            rating,
            review_text,
            timestamp: env.ledger().timestamp(),
        };

        let reviews_key = String::from_str(env, "professional_reviews");
        let mut reviews: Vec<ProfessionalReview> = env.storage().instance()
            .get(&reviews_key)
            .unwrap_or(Vec::new(env));
        reviews.push_back(review);
        env.storage().instance().set(&reviews_key, &reviews);

        env.events().publish(
            (Symbol::new(env, "professional_review_submitted"),),
            (professional_id, reviewer_id, rating),
        );

        Ok(())
    }

    pub fn get_professional_reviews(
        env: &Env,
        professional_id: Address,
        limit: Option<u32>,
    ) -> Vec<ProfessionalReview> {
        let reviews_key = String::from_str(env, "professional_reviews");
        let mut reviews: Vec<ProfessionalReview> = env.storage().instance()
            .get(&reviews_key)
            .unwrap_or(Vec::new(env));

        if let Some(limit) = limit {
            if reviews.len() > limit {
                let start = reviews.len() - limit;
                let mut result = Vec::new(env);
                for i in start..reviews.len() {
                    result.push_back(reviews.get(i).unwrap());
                }
                reviews = result;
            }
        }

        reviews
    }

    pub fn verify_professional(
        env: &Env,
        professional_id: Address,
        verifier: Address,
        verified: bool,
    ) -> Result<(), Error> {
        // In a real implementation, only authorized verifiers can do this
        let mut professional: MentalHealthProfessional = env.storage().instance()
            .get(&professional_id)
            .ok_or(Error::ProfessionalNotFound)?;

        professional.verified = verified;
        env.storage().instance().set(&professional_id, &professional);

        env.events().publish(
            (Symbol::new(env, "professional_verified"),),
            (professional_id, verified),
        );

        Ok(())
    }

    pub fn match_patient_to_professional(
        env: &Env,
        patient_id: Address,
        preferences: MatchingPreferences,
    ) -> Vec<MatchingResult> {
        // Simplified matching algorithm
        // In a real implementation, this would use complex algorithms

        let mut results = Vec::new(env);

        // Get patient's profile and needs
        let patient_profile: UserProfile = env.storage().instance()
            .get(&patient_id)
            .unwrap();

        // This is a placeholder - in reality, we'd need to search through all professionals
        // For now, return empty results

        results
    }

    pub fn schedule_appointment(
        env: &Env,
        patient_id: Address,
        professional_id: Address,
        appointment_time: u64,
        appointment_type: String,
        notes: String,
    ) -> Result<u64, Error> {
        // Verify professional exists
        let _: MentalHealthProfessional = env.storage().instance()
            .get(&professional_id)
            .ok_or(Error::ProfessionalNotFound)?;

        let appointment_id = env.ledger().timestamp() as u64;

        let appointment = Appointment {
            appointment_id,
            patient_id,
            professional_id,
            appointment_time,
            appointment_type,
            status: AppointmentStatus::Scheduled,
            notes,
            created_timestamp: env.ledger().timestamp(),
        };

        // Store appointment
        let appointments_key = String::from_str(env, "professional_appointments");
        let mut appointments: Vec<Appointment> = env.storage().instance()
            .get(&appointments_key)
            .unwrap_or(Vec::new(env));
        appointments.push_back(appointment);
        env.storage().instance().set(&appointments_key, &appointments);

        env.events().publish(
            (Symbol::new(env, "appointment_scheduled"),),
            (appointment_id, patient_id, professional_id),
        );

        Ok(appointment_id)
    }

    pub fn get_professional_appointments(
        env: &Env,
        professional_id: Address,
        start_time: u64,
        end_time: u64,
    ) -> Vec<Appointment> {
        let appointments_key = String::from_str(env, "professional_appointments");
        let appointments: Vec<Appointment> = env.storage().instance()
            .get(&appointments_key)
            .unwrap_or(Vec::new(env));

        appointments.iter()
            .filter(|apt| apt.appointment_time >= start_time && apt.appointment_time <= end_time)
            .collect()
    }
}

#[contracttype]
#[derive(Clone)]
pub struct ProfessionalReview {
    pub reviewer_id: Address,
    pub rating: u32,
    pub review_text: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct MatchingPreferences {
    pub specializations: Vec<String>,
    pub languages: Vec<String>,
    pub insurance: Option<String>,
    pub max_distance: Option<u32>,
    pub availability_preferences: Vec<String>,
}

#[contracttype]
#[derive(Clone)]
pub struct MatchingResult {
    pub professional_id: Address,
    pub match_score: f32,
    pub reasons: Vec<String>,
}

#[contracttype]
#[derive(Clone)]
pub struct Appointment {
    pub appointment_id: u64,
    pub patient_id: Address,
    pub professional_id: Address,
    pub appointment_time: u64,
    pub appointment_type: String,
    pub status: AppointmentStatus,
    pub notes: String,
    pub created_timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    Completed,
    Cancelled,
    NoShow,
}