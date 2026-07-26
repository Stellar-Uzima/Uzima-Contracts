#[cfg(test)]
mod tests {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Bytes, Env, Map, String as SorobanString, Vec};

    #[test]
    fn test_concurrency_stats() {
        let env = Env::default();
        let stats = DicomwebServicesContract::get_concurrency_stats(env);
        assert_eq!(stats.active_requests, 0);
        assert_eq!(stats.total_requests, 0);
    }

    #[test]
    fn test_list_studies_empty() {
        let env = Env::default();
        let studies = DicomwebServicesContract::list_studies(env.clone());
        assert_eq!(studies.len(), 0);
    }

    #[test]
    fn test_get_study_not_found() {
        let env = Env::default();
        let study_uid = SorobanString::from_str(&env, "1.2.3.4.5");
        let result = DicomwebServicesContract::get_study(env, study_uid);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_series_not_found() {
        let env = Env::default();
        let study_uid = SorobanString::from_str(&env, "1.2.3.4.5");
        let series_uid = SorobanString::from_str(&env, "1.2.3.4.5.1");
        let result = DicomwebServicesContract::get_series(env, study_uid, series_uid);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_get_miss() {
        let env = Env::default();
        let key = BytesN::<32>::from_array(&env, &[0u8; 32]);
        let result = DicomwebServicesContract::cache_get(env, key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::CacheMiss);
    }

    #[test]
    fn test_query_params_creation() {
        let env = Env::default();
        let params = DicomwebQueryParams {
            study_instance_uid: None,
            series_instance_uid: None,
            sop_instance_uid: None,
            patient_id: None,
            patient_name: None,
            modality: None,
            study_date_from: None,
            study_date_to: None,
            body_part: None,
            limit: 100,
            offset: 0,
        };
        assert_eq!(params.limit, 100);
        assert_eq!(params.offset, 0);
    }

    #[test]
    fn test_dicom_study_creation() {
        let env = Env::default();
        let study = DicomwebStudy {
            study_instance_uid: SorobanString::from_str(&env, "1.2.3.4.5"),
            patient_id: SorobanString::from_str(&env, "PAT001"),
            patient_name: SorobanString::from_str(&env, "John Doe"),
            study_date: 1234567890,
            study_description: SorobanString::from_str(&env, "CT Chest"),
            modalities_in_study: Vec::new(&env),
            number_of_series: 0,
            number_of_instances: 0,
            json_metadata: DicomJsonObject {
                attributes: Map::new(&env),
            },
        };
        assert_eq!(study.study_date, 1234567890);
        assert_eq!(study.number_of_series, 0);
    }

    #[test]
    fn test_transfer_syntax_enum() {
        // Verify transfer syntax enum values
        let ts1 = TransferSyntax::ExplicitVrLittleEndian;
        let ts2 = TransferSyntax::ImplicitVrLittleEndian;
        let ts3 = TransferSyntax::Jpeg2000Lossless;
        assert_eq!(ts1, TransferSyntax::ExplicitVrLittleEndian);
        assert_eq!(ts2, TransferSyntax::ImplicitVrLittleEndian);
        assert_eq!(ts3, TransferSyntax::Jpeg2000Lossless);
    }

    #[test]
    fn test_error_enum_values() {
        // Verify error codes
        assert_eq!(Error::AlreadyInitialized as u32, 1);
        assert_eq!(Error::NotInitialized as u32, 2);
        assert_eq!(Error::NotAuthorized as u32, 3);
        assert_eq!(Error::ContractPaused as u32, 4);
        assert_eq!(Error::InvalidInput as u32, 5);
    }

    #[test]
    fn test_cache_entry_creation() {
        let env = Env::default();
        let key = BytesN::<32>::from_array(&env, &[1u8; 32]);
        let data = Bytes::from_array(&env, &[42u8; 32]);
        let entry = CacheEntry {
            key: key.clone(),
            data: data.clone(),
            created_at: 1000,
            expires_at: 5000,
            hit_count: 0,
        };
        assert_eq!(entry.created_at, 1000);
        assert_eq!(entry.expires_at, 5000);
        assert_eq!(entry.hit_count, 0);
    }

    #[test]
    fn test_concurrency_tracker_creation() {
        let tracker = ConcurrencyTracker {
            active_requests: 0,
            total_requests: 0,
            last_reset: 1000,
        };
        assert_eq!(tracker.active_requests, 0);
        assert_eq!(tracker.total_requests, 0);
        assert_eq!(tracker.last_reset, 1000);
    }
}
