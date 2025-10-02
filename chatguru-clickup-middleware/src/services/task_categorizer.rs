use std::collections::HashMap;
use once_cell::sync::Lazy;

/// IDs dos campos personalizados do ClickUp
pub const FIELD_CATEGORIA: &str = "eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a";
pub const FIELD_SUBCATEGORIA: &str = "5333c095-eb40-4a5a-b0c2-76bfba4b1094";
pub const FIELD_ESTRELAS: &str = "83afcb8c-2866-498f-9c62-8ea9666b104b";

/// Mapa: Subcategoria → Estrelas (tabela HTML)
static SUBCATEGORIA_ESTRELAS: Lazy<HashMap<&'static str, u8>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Agendamentos
    m.insert("Consultas", 1);
    m.insert("Exames", 1);
    m.insert("Veterinário/Petshop (Consultas/Exames/Banhos/Tosas)", 1);
    m.insert("Vacinas", 1);
    m.insert("Manicure", 1);
    m.insert("Cabeleleiro", 1);

    // Compras
    m.insert("Shopper", 2);
    m.insert("Mercados", 1);
    m.insert("Presentes", 1);
    m.insert("Petshop", 1);
    m.insert("Papelaria", 1);
    m.insert("Farmácia", 2);
    m.insert("Ingressos", 2);
    m.insert("Móveis e Eletros", 2);
    m.insert("Itens pessoais e da casa", 2);

    // Documentos
    m.insert("CIN", 1);
    m.insert("Certificado Digital", 2);
    m.insert("Documento de Vacinação (BR/Iternacional)", 1);
    m.insert("Seguros Carro/Casa/Viagem (anual)", 2);
    m.insert("Assinatura Digital", 1);
    m.insert("Contratos/Procurações", 1);
    m.insert("Cidadanias", 4);
    m.insert("Vistos e Vistos Eletrônicos", 2);
    m.insert("Passaporte", 1);
    m.insert("CNH", 1);
    m.insert("Averbações", 1);
    m.insert("Certidões", 1);

    // Lazer
    m.insert("Reserva de restaurantes/bares", 1);
    m.insert("Planejamento de festas", 4);
    m.insert("Pesquisa de passeios/eventos (BR)", 3);
    m.insert("Fornecedores no exterior (passeios, fotógrafos)", 2);

    // Logística
    m.insert("Corrida de motoboy", 1);
    m.insert("Motoboy + Correios e envios internacionais", 1);
    m.insert("Lalamove", 1);
    m.insert("Corridas com Taxistas", 1);
    m.insert("Transporte Urbano (Uber/99)", 1);

    // Viagens
    m.insert("Passagens Aéreas", 2);
    m.insert("Hospedagens", 2);
    m.insert("Compra de Assentos e Bagagens", 1);
    m.insert("Passagens de Ônibus", 1);
    m.insert("Passagens de Trem", 2);
    m.insert("Checkins (Early/Late)", 1);
    m.insert("Extravio de Bagagens", 2);
    m.insert("Seguro Viagem (Temporário)", 1);
    m.insert("Transfer", 2);
    m.insert("Programa de Milhagem", 1);
    m.insert("Gestão de Contas (CIAs Aereas)", 1);
    m.insert("Aluguel de Carro/Ônibus e Vans", 2);
    m.insert("Roteiro de Viagens", 3);

    // Plano de Saúde
    m.insert("Reembolso Médico", 2);
    m.insert("Extrato para IR", 1);
    m.insert("Prévia de Reembolso", 1);
    m.insert("Contestações", 1);
    m.insert("Autorizações", 1);
    m.insert("Contratações/Cancelamentos", 2);

    // Agenda
    m.insert("Gestão de Agenda", 1);
    m.insert("Criação e envio de invites", 1);

    // Financeiro
    m.insert("Emissão de NF", 1);
    m.insert("Rotina de Pagamentos", 1);
    m.insert("Emissão de boletos", 1);
    m.insert("Conciliação Bancária", 2);
    m.insert("Planilha de Gastos/Pagamentos", 4);
    m.insert("Encerramento e Abertura de CNPJ", 2);
    m.insert("Imposto de Renda", 1);
    m.insert("Emissão de Guias de Imposto (DARF, DAS, DIRF, GPS)", 1);

    // Assuntos Pessoais
    m.insert("Mudanças", 3);
    m.insert("Troca de titularidade", 1);
    m.insert("Assuntos do Carro/Moto", 1);
    m.insert("Internet e TV por Assinatura", 1);
    m.insert("Contas de Consumo", 1);
    m.insert("Anúncio de Vendas Online (Itens, eletros. móveis)", 3);
    m.insert("Assuntos Escolares e Professores Particulares", 1);
    m.insert("Academia e Cursos Livres", 1);
    m.insert("Telefone", 1);
    m.insert("Assistência Técnica", 1);
    m.insert("Consertos na Casa", 1);

    // Atividades Corporativas
    m.insert("Financeiro/Contábil", 1);
    m.insert("Atendimento ao Cliente", 1);
    m.insert("Gestão de Planilhas e Emails", 4);
    m.insert("Documentos/Contratos e Assinaturas", 1);
    m.insert("Gestão de Agenda (Corporativa)", 1);
    m.insert("Recursos Humanos", 1);
    m.insert("Gestão de Estoque", 1);
    m.insert("Compras/vendas", 1);
    m.insert("Fornecedores", 2);

    // Gestão de Funcionário
    m.insert("eSocial", 1);
    m.insert("Contratações e Desligamentos", 1);
    m.insert("DIRF", 1);
    m.insert("Férias", 1);

    m
});

/// Categorização baseada em palavras-chave
#[derive(Debug, Clone)]
pub struct Categorization {
    pub categoria: String,
    pub subcategoria: String,
    pub estrelas: u8,
}

/// Analisa o texto da tarefa e retorna a categorização automática
pub fn categorize_task(text: &str) -> Option<Categorization> {
    let text_lower = text.to_lowercase();

    // Logística
    if text_lower.contains("motoboy") {
        return Some(categorize("Logística", "Corrida de motoboy"));
    }
    if text_lower.contains("sedex") || text_lower.contains("correio") {
        return Some(categorize("Logística", "Motoboy + Correios e envios internacionais"));
    }
    if text_lower.contains("lalamove") {
        return Some(categorize("Logística", "Lalamove"));
    }
    if text_lower.contains("uber") || text_lower.contains("99") {
        return Some(categorize("Logística", "Transporte Urbano (Uber/99)"));
    }
    if text_lower.contains("taxista") {
        return Some(categorize("Logística", "Corridas com Taxistas"));
    }
    if text_lower.contains("entrega") || text_lower.contains("retirada") {
        return Some(categorize("Logística", "Corrida de motoboy"));
    }

    // Plano de Saúde
    if text_lower.contains("reembolso") || text_lower.contains("bradesco saúde") || text_lower.contains("plano de saúde") {
        return Some(categorize("Plano de Saúde", "Reembolso Médico"));
    }

    // Compras
    if text_lower.contains("mercado") {
        return Some(categorize("Compras", "Mercados"));
    }
    if text_lower.contains("farmácia") {
        return Some(categorize("Compras", "Farmácia"));
    }
    if text_lower.contains("presente") {
        return Some(categorize("Compras", "Presentes"));
    }
    if text_lower.contains("shopper") {
        return Some(categorize("Compras", "Shopper"));
    }
    if text_lower.contains("papelaria") {
        return Some(categorize("Compras", "Papelaria"));
    }
    if text_lower.contains("petshop") {
        return Some(categorize("Compras", "Petshop"));
    }
    if text_lower.contains("ingresso") {
        return Some(categorize("Compras", "Ingressos"));
    }

    // Assuntos Pessoais
    if text_lower.contains("troca") {
        return Some(categorize("Assuntos Pessoais", "Troca de titularidade"));
    }
    if text_lower.contains("internet") {
        return Some(categorize("Assuntos Pessoais", "Internet e TV por Assinatura"));
    }
    if text_lower.contains("telefone") {
        return Some(categorize("Assuntos Pessoais", "Telefone"));
    }
    if text_lower.contains("conserto") {
        return Some(categorize("Assuntos Pessoais", "Consertos na Casa"));
    }
    if text_lower.contains("assistência") {
        return Some(categorize("Assuntos Pessoais", "Assistência Técnica"));
    }

    // Financeiro
    if text_lower.contains("pagamento") {
        return Some(categorize("Financeiro", "Rotina de Pagamentos"));
    }
    if text_lower.contains("boleto") {
        return Some(categorize("Financeiro", "Emissão de boletos"));
    }
    if text_lower.contains("nota fiscal") {
        return Some(categorize("Financeiro", "Emissão de NF"));
    }

    // Viagens
    if text_lower.contains("passagem") {
        return Some(categorize("Viagens", "Passagens Aéreas"));
    }
    if text_lower.contains("hospedagem") || text_lower.contains("hotel") {
        return Some(categorize("Viagens", "Hospedagens"));
    }
    if text_lower.contains("check in") {
        return Some(categorize("Viagens", "Checkins (Early/Late)"));
    }
    if text_lower.contains("bagagem") {
        return Some(categorize("Viagens", "Extravio de Bagagens"));
    }

    // Agendamentos
    if text_lower.contains("consulta") {
        return Some(categorize("Agendamentos", "Consultas"));
    }
    if text_lower.contains("exame") {
        return Some(categorize("Agendamentos", "Exames"));
    }
    if text_lower.contains("vacina") {
        return Some(categorize("Agendamentos", "Vacinas"));
    }
    if text_lower.contains("manicure") {
        return Some(categorize("Agendamentos", "Manicure"));
    }
    if text_lower.contains("cabeleireiro") {
        return Some(categorize("Agendamentos", "Cabeleleiro"));
    }

    // Lazer
    if text_lower.contains("restaurante") || text_lower.contains("reserva") {
        return Some(categorize("Lazer", "Reserva de restaurantes/bares"));
    }
    if text_lower.contains("festa") {
        return Some(categorize("Lazer", "Planejamento de festas"));
    }

    // Documentos
    if text_lower.contains("passaporte") {
        return Some(categorize("Documentos", "Passaporte"));
    }
    if text_lower.contains("cnh") {
        return Some(categorize("Documentos", "CNH"));
    }
    if text_lower.contains("cidadania") {
        return Some(categorize("Documentos", "Cidadanias"));
    }
    if text_lower.contains("visto") {
        return Some(categorize("Documentos", "Vistos e Vistos Eletrônicos"));
    }
    if text_lower.contains("certidão") {
        return Some(categorize("Documentos", "Certidões"));
    }
    if text_lower.contains("contrato") {
        return Some(categorize("Documentos", "Contratos/Procurações"));
    }

    None
}

fn categorize(categoria: &str, subcategoria: &str) -> Categorization {
    let estrelas = SUBCATEGORIA_ESTRELAS.get(subcategoria).copied().unwrap_or(1);

    Categorization {
        categoria: categoria.to_string(),
        subcategoria: subcategoria.to_string(),
        estrelas,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_motoboy() {
        let result = categorize_task("Envio de reembolso via motoboy").unwrap();
        assert_eq!(result.categoria, "Logística");
        assert_eq!(result.subcategoria, "Corrida de motoboy");
        assert_eq!(result.estrelas, 1);
    }

    #[test]
    fn test_categorize_reembolso() {
        let result = categorize_task("Reembolso médico bradesco saúde").unwrap();
        assert_eq!(result.categoria, "Plano de Saúde");
        assert_eq!(result.subcategoria, "Reembolso Médico");
        assert_eq!(result.estrelas, 2);
    }

    #[test]
    fn test_categorize_festa() {
        let result = categorize_task("Planejamento de festa de aniversário").unwrap();
        assert_eq!(result.categoria, "Lazer");
        assert_eq!(result.subcategoria, "Planejamento de festas");
        assert_eq!(result.estrelas, 4);
    }
}
