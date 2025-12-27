# Sistema de Votación en ink!

Contrato inteligente de votación desarrollado para el ecosistema Polkadot.

## Funcionalidades
- Gestión de propuestas por el administrador.
- Votación única por usuario.
- Validación de errores (propuesta inexistente, doble voto).
- Emisión de eventos para auditoría.

## Tests
El contrato incluye 6 tests unitarios que validan:
1. Inicialización y asignación de Owner.
2. Registro de votos exitoso.
3. Reversión de doble voto.
4. Reversión de propuesta inexistente.
5. Creación y consulta de propuestas.
6. Emisión de eventos.

## Comandos
- Ejecutar tests: cargo +nightly test
- Compilar: cargo +nightly contract build

## Autor
Lautarrro- UNLP