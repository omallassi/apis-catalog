
#[derive(Debug, Clone)]
pub struct  Proto3{
    spec: String,
}
impl Proto3 {
    pub fn new(val: &str) -> Proto3 {
        Proto3 {
            spec: String::from(val),
        }
    }
}
impl crate::app::dao::catalog::handlers::SpecHandler for Proto3{

    fn get_version(&self) -> String {
        "you should use prost-types crate".to_string()
    }

    fn get_description(&self) -> String {
        "you should use prost-types crate".to_string()
    }

    fn get_paths_len(&self) -> usize {
        2
    }

    fn get_title(&self) -> String {
        "you should use prost-types crate".to_string()
    }

    fn get_paths(&self) -> Vec<crate::app::dao::catalog::handlers::Path> {
        Vec::new()
    }
}

// impl std::fmt::Debug for Proto3 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "Debug not implemented"
//         )
//     }
// }

// impl Clone for Proto3 {
//     fn clone(&self) -> Self {
//         Proto3 {
//             spec: self.spec.clone(),
//         }
//     }
// }

#[cfg(test)]
pub mod tests {
    use crate::app::dao::catalog::handlers::SpecHandler;

    #[test]
    fn test_play_with_trait(){
        //TODO find a real one
        let proto_spec = "
        syntax=\"proto3\";
        // Enable custom Marshal method.
        option (gogoproto.marshaler_all) = true;
        // Enable custom Unmarshal method.
        option (gogoproto.unmarshaler_all) = true;
        // Enable custom Size method (Required by Marshal and Unmarshal).
        option (gogoproto.sizer_all) = true;
        // Enable registration with golang/protobuf for the grpc-gateway.
        option (gogoproto.goproto_registration) = true;
        // Enable generation of XXX_MessageName methods for grpc-go/status.
        option (gogoproto.messagename_all) = true;

        service UserService {
            rpc AddUser(User) returns (google.protobuf.Empty) {
            }
            rpc ListUsers(ListUsersRequest) returns (stream User) {
            }
            rpc ListUsersByRole(UserRole) returns (stream User) {
            }
            rpc UpdateUser(UpdateUserRequest) returns (User) {
            }
        }";

        let spec = crate::app::dao::catalog::handlers::implem::proto::Proto3::new(proto_spec);
        assert_eq!(spec.get_version(), "you should use prost-types crate");
    }

}