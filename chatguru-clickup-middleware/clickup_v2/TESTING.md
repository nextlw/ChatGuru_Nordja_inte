# Estratégia de Testes - ClickUp v2 Rust Crate

## Visão Geral

Este documento descreve a estratégia completa de testes para o crate ClickUp v2, incluindo tipos de testes, ferramentas utilizadas, e como executá-los.

## Estrutura de Testes

### 1. Testes Unitários

Localizados dentro de cada módulo no diretório `src/`, testam funcionalidades isoladas.

- **Localização**: `src/**/*.rs` (dentro de `#[cfg(test)]` modules)
- **Cobertura**:
  - Módulo `config/env.rs`: Configuração e variáveis de ambiente
  - Módulo `auth/oauth.rs`: Fluxo OAuth2
  - Módulo `error/auth_error.rs`: Tratamento de erros
  - Módulo `client/api.rs`: Cliente HTTP

### 2. Testes de Integração

Localizados no diretório `tests/`, testam a interação entre módulos.

- **Localização**: `tests/`
- **Arquivos**:
  - `api_integration_tests.rs`: Testes de API completos
  - `oauth_integration_tests.rs`: Fluxo OAuth2 completo
  - `env_config_tests.rs`: Configuração de ambiente
  - `error_handling_tests.rs`: Cenários de erro

### 3. Mocks

Utilitários para simular respostas HTTP em testes.

- **Localização**: `tests/mocks/`
- **Propósito**: Simular respostas da API ClickUp sem fazer chamadas reais

### 4. Benchmarks

Testes de performance para identificar gargalos.

- **Localização**: `benches/`
- **Arquivo**: `api_benchmarks.rs`

## Ferramentas e Dependências

### Dependências de Teste (dev-dependencies)

```toml
[dev-dependencies]
tokio-test = "0.4"          # Framework de testes assíncronos
mockito = "1.2"             # Mock de servidor HTTP
wiremock = "0.6"            # Mock avançado de HTTP
pretty_assertions = "1.4"    # Assertions mais legíveis
claim = "0.5"               # Assertions adicionais
fake = "2.9"                # Geração de dados de teste
quickcheck = "1.0"          # Testes baseados em propriedades
insta = "1.34"              # Testes de snapshot
temp-env = "0.3"            # Manipulação de variáveis de ambiente em testes
criterion = "0.5"           # Framework de benchmark
```

## Como Executar os Testes

### Usando Cargo (Manual)

```bash
# Executar todos os testes
cargo test

# Executar apenas testes unitários
cargo test --lib

# Executar apenas testes de integração
cargo test --test '*'

# Executar testes com output verboso
cargo test --verbose

# Executar testes de um módulo específico
cargo test config::

# Executar um teste específico
cargo test test_oauth_flow_creation

# Executar testes em paralelo (padrão) ou sequencial
cargo test -- --test-threads=1

# Executar testes ignorados
cargo test -- --ignored
```

### Usando Make (Automatizado)

```bash
# Ver todos os comandos disponíveis
make help

# Executar todos os testes
make test

# Executar apenas testes unitários
make test-unit

# Executar apenas testes de integração
make test-integration

# Executar testes de documentação
make test-doc

# Executar todos os testes incluindo documentação
make test-all

# Executar testes e gerar relatório de cobertura
make coverage

# Executar benchmarks
make bench

# Pipeline completo de CI
make ci

# Preparar para commit (formatar, lint, testar)
make pre-commit
```

## Cobertura de Código

### Gerando Relatório de Cobertura

```bash
# Instalar tarpaulin (apenas uma vez)
cargo install cargo-tarpaulin

# Gerar relatório HTML
make coverage

# Gerar e abrir relatório no navegador
make coverage-open

# Gerar relatório XML para CI
cargo tarpaulin --out xml
```

### Interpretando a Cobertura

- **Meta**: > 80% de cobertura geral
- **Crítico**: > 90% em módulos de autenticação e erro
- **Aceitável**: > 70% em módulos auxiliares

## Benchmarks

### Executando Benchmarks

```bash
# Executar todos os benchmarks
make bench

# Salvar baseline para comparação
make bench-save

# Comparar com baseline salvo
make bench-compare

# Executar benchmark específico
cargo bench benchmark_client_creation
```

### Benchmarks Disponíveis

- `benchmark_client_creation`: Criação de cliente
- `benchmark_env_manager_load`: Carregamento de configurações
- `benchmark_oauth_flow_creation`: Criação de fluxo OAuth
- `benchmark_api_call`: Chamadas de API
- `benchmark_json_parsing`: Parsing de JSON
- `benchmark_url_construction`: Construção de URLs
- `benchmark_validation`: Validações
- `benchmark_concurrent_api_calls`: Chamadas concorrentes

## Mocking

### Usando Mockito

```rust
use mockito::{mock, Matcher};

#[tokio::test]
async fn test_api_call() {
    let _m = mock("GET", "/user")
        .with_status(200)
        .with_body(r#"{"id": 123}"#)
        .create();

    // Seu teste aqui
}
```

### Usando os Mocks Utilitários

```rust
use crate::mocks::{mock_authenticated_user, MockServer};

#[tokio::test]
async fn test_complete_flow() {
    let _mocks = MockServer::new()
        .with_authenticated_user()
        .with_teams()
        .with_spaces("123")
        .build();

    // Seu teste aqui
}
```

## Variáveis de Ambiente para Testes

Os testes requerem as seguintes variáveis de ambiente:

```bash
CLICKUP_CLIENT_ID=test_client_id
CLICKUP_CLIENT_SECRET=test_client_secret
```

### Criar arquivo .env de teste

```bash
make test-env
```

Isso criará um arquivo `.env.test` com valores padrão para testes.

## CI/CD

### GitHub Actions

O projeto usa GitHub Actions para CI/CD. Os workflows estão em `.github/workflows/`:

- **tests.yml**: Pipeline principal de testes

#### Jobs do CI

1. **Test**: Executa testes em múltiplas versões do Rust (stable, beta, nightly)
2. **Coverage**: Gera e envia cobertura para Codecov
3. **Integration**: Executa testes de integração com mock server
4. **Benchmark**: Executa e salva resultados de benchmarks

### Executar CI Localmente

```bash
# Simular pipeline CI completo
make ci
```

## Estratégias de Teste

### 1. Test-Driven Development (TDD)

Recomendamos seguir TDD para novas funcionalidades:

1. Escrever teste que falha
2. Implementar código mínimo para passar
3. Refatorar mantendo testes verdes

### 2. Testes de Regressão

- Sempre adicionar testes para bugs corrigidos
- Manter suite de testes de regressão em `tests/regression/`

### 3. Testes de Propriedades

Usar QuickCheck para testes baseados em propriedades:

```rust
use quickcheck_macros::quickcheck;

#[quickcheck]
fn prop_url_construction(endpoint: String) -> bool {
    // Propriedade a ser testada
    true
}
```

### 4. Testes de Snapshot

Usar Insta para testes de snapshot de respostas complexas:

```rust
use insta::assert_snapshot;

#[test]
fn test_complex_response() {
    let response = get_complex_data();
    assert_snapshot!(response);
}
```

## Boas Práticas

### 1. Isolamento de Testes

- Usar `temp-env` para isolar variáveis de ambiente
- Criar novos mocks para cada teste
- Não depender de ordem de execução

### 2. Nomenclatura

- Testes unitários: `test_<função>_<cenário>`
- Testes de integração: `test_<fluxo>_<resultado>`
- Benchmarks: `benchmark_<operação>`

### 3. Organização

- Agrupar testes relacionados em módulos
- Usar `#[cfg(test)]` para código de teste
- Separar fixtures e helpers em módulos próprios

### 4. Assertions

- Preferir `assert_eq!` e `assert_ne!` sobre `assert!`
- Usar `pretty_assertions` para comparações complexas
- Adicionar mensagens descritivas em assertions

### 5. Testes Assíncronos

```rust
#[tokio::test]
async fn test_async_operation() {
    // Use tokio::test para testes assíncronos
}
```

## Debugging de Testes

### Executar teste específico com output

```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Usar debugger

```bash
# Com lldb
rust-lldb target/debug/deps/test_binary

# Com gdb
rust-gdb target/debug/deps/test_binary
```

### Variáveis de ambiente úteis

```bash
# Mostrar backtrace completo em erros
RUST_BACKTRACE=1 cargo test

# Mostrar backtrace completo
RUST_BACKTRACE=full cargo test

# Debug de macros
RUST_LOG=debug cargo test
```

## Checklist de Testes

Antes de fazer commit ou criar PR, verifique:

- [ ] Todos os testes passam (`make test`)
- [ ] Código está formatado (`make fmt`)
- [ ] Sem warnings do clippy (`make lint`)
- [ ] Cobertura adequada para novo código
- [ ] Testes de integração para novas features
- [ ] Documentação atualizada
- [ ] Benchmarks não regressaram significativamente

## Resolução de Problemas Comuns

### 1. Testes falhando por timeout

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[timeout(Duration::from_secs(10))]
async fn test_with_timeout() {
    // teste
}
```

### 2. Conflitos de porta em testes

Use portas dinâmicas ou diferentes para cada teste:

```rust
let port = 8888 + (test_id % 1000);
```

### 3. Testes intermitentes

- Adicionar delays apropriados
- Usar mocks determinísticos
- Verificar condições de corrida

### 4. Variáveis de ambiente interferindo

Sempre usar `temp-env` para isolar:

```rust
temp_env::with_var("VAR", Some("value"), || {
    // teste isolado
});
```

## Métricas de Qualidade

### Targets de Qualidade

- **Cobertura de Código**: > 80%
- **Tempo de Execução**: < 30 segundos para suite completa
- **Taxa de Sucesso**: 100% em CI
- **Complexidade Ciclomática**: < 10 por função

### Monitoramento

- Codecov para tracking de cobertura
- GitHub Actions para CI
- Criterion para tracking de performance

## Contribuindo com Testes

Ao contribuir com o projeto:

1. Adicione testes para qualquer nova funcionalidade
2. Mantenha cobertura existente
3. Siga as convenções de nomenclatura
4. Documente comportamentos complexos
5. Execute suite completa antes de PR

## Recursos Adicionais

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [Mockito Docs](https://docs.rs/mockito/)
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [Cargo Tarpaulin](https://github.com/xd009642/tarpaulin)