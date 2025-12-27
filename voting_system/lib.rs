#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod voting_system {

    use ink::prelude::string::String;
    use ink::storage::Mapping;

    /// =========================
    /// MODELO
    /// =========================
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    //  ERRORES
    /// =========================
    pub enum Error {
        OnlyOwnerCanPerformAction,
        ProposalDoesNotExist,
        AlreadyVoted,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Proposal {
        pub description: String,
        pub votes: u32,
    }
    // EVENTOS
    // =========================
    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        pub id: u32,
        pub title: String,
    }

    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        pub proposal_id: u32,
        #[ink(topic)]
        pub voter: AccountId,
    }

    /// STORAGE
    /// =========================
    #[ink(storage)]
    pub struct VotingSystem {
        proposals: Mapping<u32, Proposal>,
        voters: Mapping<(u32, AccountId), bool>,
        proposal_count: u32,
        owner: AccountId,
    }

  
    /// IMPLEMENTACION
    /// =========================
    impl VotingSystem {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                proposals: Mapping::default(),
                voters: Mapping::default(),
                proposal_count: 0,
                owner: Self::env().caller(),
            }
        }

        /// Crear una propuesta
        #[ink(message)]
        pub fn create_proposal(&mut self, title: String) -> Result<u32, Error> {
            //Valida que el caller sea el owner
            if self.env().caller() != self.owner {
                return Err(Error::OnlyOwnerCanPerformAction);
            }
            //Asignar ID
            let id = self.proposal_count;
            // crear la propuesta
            let proposal = Proposal {
                description: title.clone(),
                votes: 0,
            };
            // Almacenar la propuesta
            self.proposals.insert(id, &proposal);
            self.proposal_count = self.proposal_count.saturating_add(1);
            //Emite el evento
            self.env().emit_event(ProposalCreated { id, title });
        
            Ok(id)
        }

        /// Votar una propuesta (una vez por cuenta)
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u32) -> Result<(), Error> {
            let caller = self.env().caller();

           // Verificar existencia 
            let mut proposal = self.proposals.get(proposal_id).ok_or(Error::ProposalDoesNotExist)?;

            // Verificar que no haya votado antes 
            if self.voters.get((proposal_id, caller)).unwrap_or(false) {
                return Err(Error::AlreadyVoted);
            }

            // Registrar voto
            proposal.votes = proposal.votes.saturating_add(1);
            self.proposals.insert(proposal_id, &proposal);
            self.voters.insert((proposal_id, caller), &true);

            // Emitir evento 
            self.env().emit_event(VoteCast { proposal_id, voter: caller });

            Ok(())
        }

        /// Obtener una propuesta
        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Result<(String, u32), Error> {
            let proposal = self.proposals.get(proposal_id).ok_or(Error::ProposalDoesNotExist)?;
            Ok((proposal.description, proposal.votes))
        }

        /// Cantidad total de propuestas
        #[ink(message)]
        pub fn total_proposals(&self) -> u32 {
            self.proposal_count
        }
    }

    /// =========================
    /// TESTS
    /// =========================
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        /// Función auxiliar para simular quién firma la transacción
        fn set_caller(account: AccountId) {
            test::set_caller::<ink::env::DefaultEnvironment>(account);
        }

        #[ink::test]
        fn test_inicializacion_y_acceso_owner() {
            let mut contract = VotingSystem::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            
            //  Verificar inicializacion
            assert_eq!(contract.total_proposals(), 0);

            // Verificar control de acceso para el owner 
            set_caller(accounts.bob); // Bob intenta crear
            let res = contract.create_proposal(String::from("Falla"));
            assert_eq!(res, Err(Error::OnlyOwnerCanPerformAction));
        }

        #[ink::test]
        fn test_creacion_propuestas_y_consulta() {
            // Crear contrato y establecer caller como owner
            let mut contract = VotingSystem::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            set_caller(accounts.alice); // Alice es owner por defecto en el constructor

            // Crear multiples propuestas 
            assert_eq!(contract.create_proposal(String::from("Propuestarda0")), Ok(0));
            assert_eq!(contract.create_proposal(String::from("Propuestarda00")), Ok(1));
            // Verificar conteo de propuestas
            assert_eq!(contract.total_proposals(), 2);

            // Verificar datos públicos 
            let proposal = contract.get_proposal(0).unwrap();
            assert_eq!(proposal.0, "Propuestarda0"); // .0 es el descripcion
            assert_eq!(proposal.1, 0);             // .1 son los votos iniciales
        }

        #[ink::test]
        fn test_registro_votos_exitoso() {
            // Crear contrato y propuesta
            let mut contract = VotingSystem::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            // Owner crea propuesta
            set_caller(accounts.alice);
            contract.create_proposal(String::from("Propuestarda1")).unwrap();

            // Distintos usuarios votan
            set_caller(accounts.bob);
            assert!(contract.vote(0).is_ok());
            
            set_caller(accounts.charlie);
            assert!(contract.vote(0).is_ok());
            // Verificar conteo de votos
            let (_, votos) = contract.get_proposal(0).unwrap();
            assert_eq!(votos, 2);
        }

        #[ink::test]
        fn test_reversion_doble_voto() {
            // Crear contrato y propuesta
            let mut contract = VotingSystem::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            // Owner crea propuesta
            set_caller(accounts.alice);
            contract.create_proposal(String::from("Unico Voto")).unwrap();
            // Usuario vota
            set_caller(accounts.bob);
            assert!(contract.vote(0).is_ok());
            
            // Reversion al votar dos veces 
            assert_eq!(contract.vote(0), Err(Error::AlreadyVoted));
        }

        #[ink::test]
        fn test_reversion_propuesta_inexistente() {
            let mut contract = VotingSystem::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            set_caller(accounts.bob);

            // Reversion al votar propuestas inexistentes 
            assert_eq!(contract.vote(99), Err(Error::ProposalDoesNotExist));
            assert_eq!(contract.get_proposal(99), Err(Error::ProposalDoesNotExist));
        }

        #[ink::test]
        fn test_emision_de_eventos() {
            // Crear contrato y propuesta
            let mut contract = VotingSystem::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            // Owner crea propuesta
            set_caller(accounts.alice);
            contract.create_proposal(String::from("Evento Testeardo")).unwrap();
            // Usuario vota
            set_caller(accounts.bob);
            contract.vote(0).unwrap();

            // Verificar eventos 
            let emitted_events = test::recorded_events().collect::<Vec<_>>();
            // Debería haber al menos 2 eventos emitidos
            assert!(emitted_events.len() >= 2);
        }
    
      
    
    }
}
