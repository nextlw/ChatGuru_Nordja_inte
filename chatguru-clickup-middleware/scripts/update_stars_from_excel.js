#!/usr/bin/env node

/**
 * Script para atualizar as estrelas no clickup_categories.json
 * baseado nos dados da tabela Excel API_Categorias.xlsx
 */

const fs = require('fs');
const path = require('path');

// Mapeamento de subcategorias para estrelas baseado no Excel
const SUBCATEGORY_STARS = {
  // Agendamentos (todas 1 estrela)
  "Consultas": 1,
  "Exames": 1,
  "Veterinário/Petshop (Consultas/Exames/Banhos/Tosas)": 1,
  "Vacinas": 1,
  "Manicure": 1,
  "Cabeleleiro": 1,

  // Compras
  "Shopper": 2,
  "Mercados": 1,
  "Presentes": 1,
  "Petshop": 1,
  "Papelaria": 1,
  "Farmácia": 2,
  "Ingressos": 2,
  "Móveis e Eletros": 2,
  "Itens pessoais e da casa": 2,

  // Documentos
  "CIN": 1,
  "Certificado Digital": 2,
  "Documento de Vacinação (BR/Iternacional)": 1,
  "Documento de Vacinação (BR/Internacional)": 1, // variação de escrita
  "Seguros Carro/Casa/Viagem (anual)": 2,
  "Assinatura Digital": 1,
  "Contratos/Procurações": 1,
  "Cidadanias": 4,
  "Vistos e Vistos Eletrônicos": 2,
  "Passaporte": 1,
  "CNH": 1,
  "Averbações": 1,
  "Certidões": 1,

  // Lazer
  "Reserva de restaurantes/bares": 1,
  "Planejamento de festas": 4,
  "Pesquisa de passeios/eventos (BR)": 3,
  "Fornecedores no exterior (passeios, fotógrafos)": 2,

  // Logística (todas 1 estrela)
  "Corrida de motoboy": 1,
  "Motoboy + Correios e envios internacionais": 1,
  "Lalamove": 1,
  "Corridas com Taxistas": 1,
  "Transporte Urbano (Uber/99)": 1,

  // Viagens
  "Passagens Aéreas": 2,
  "Hospedagens": 2,
  "Compra de Assentos e Bagagens": 1,
  "Passagens de Ônibus": 1,
  "Passagens de Trem": 2,
  "Checkins (Early/Late)": 1,
  "Extravio de Bagagens": 2,
  "Seguro Viagem (Temporário)": 1,
  "Transfer": 2,
  "Programa de Milhagem": 1,
  "Gestão de Contas (CIAs Aereas)": 1,
  "Aluguel de Carro/Ônibus e Vans": 2,
  "Roteiro de Viagens": 3,

  // Plano de Saúde
  "Reembolso Médico": 2,
  "Extrato para IR": 1,
  "Prévia de Reembolso": 1,
  "Contestações": 1,
  "Autorizações": 1,
  "Contratações/Cancelamentos": 2,

  // Agenda (todas 1 estrela)
  "Gestão de Agenda": 1,
  "Criação e envio de invites": 1,

  // Financeiro
  "Emissão de NF": 1,
  "Rotina de Pagamentos": 1,
  "Emissão de boletos": 1,
  "Conciliação Bancária": 2,
  "Planilha de Gastos/Pagamentos": 4,
  "Encerramento e Abertura de CNPJ": 2,
  "Imposto de Renda": 1,
  "Emissão de Guias de Imposto (DARF, DAS, DIRF, GPS)": 1,

  // Assuntos Pessoais
  "Mudanças": 3,
  "Troca de titularidade": 1,
  "Assuntos do Carro/Moto": 1,
  "Internet e TV por Assinatura": 1,
  "Contas de Consumo": 1,
  "Anúncio de Vendas Online (Itens, eletros. móveis)": 3,
  "Assuntos Escolares e Professores Particulares": 1,
  "Academia e Cursos Livres": 1,
  "Telefone": 1,
  "Assistência Técnica": 1,
  "Consertos na Casa": 1,

  // Atividades Corporativas
  "Financeiro/Contábil": 1,
  "Atendimento ao Cliente": 1,
  "Gestão de Planilhas e Emails": 4,
  "Documentos/Contratos e Assinaturas": 1,
  "Gestão de Agenda (Corporativa)": 1,
  "Recursos Humanos": 1,
  "Gestão de Estoque": 1,
  "Compras/vendas": 1,
  "Fornecedores": 2,

  // Gestão de Funcionário (todas 1 estrela)
  "eSocial": 1,
  "Contratações e Desligamentos": 1,
  "DIRF": 1,
  "Férias": 1
};

/**
 * Atualiza as estrelas no JSON do ClickUp
 */
function updateStarsInJSON() {
  const jsonPath = path.join(__dirname, '..', 'config', 'clickup_categories.json');

  console.log('📖 Lendo arquivo JSON...');
  const data = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));

  let updatedCount = 0;
  let notFoundCount = 0;
  const notFound = [];

  console.log('\n🔄 Atualizando estrelas...\n');

  // Atualizar estrelas nas subcategorias
  if (data.subcategories_map && data.subcategories_map._outros) {
    data.subcategories_map._outros.forEach(subcat => {
      const name = subcat.name;

      if (SUBCATEGORY_STARS.hasOwnProperty(name)) {
        const newStars = SUBCATEGORY_STARS[name];
        if (subcat.stars !== newStars) {
          console.log(`  ✓ "${name}": ${subcat.stars} → ${newStars} estrelas`);
          subcat.stars = newStars;
          updatedCount++;
        }
      } else {
        notFoundCount++;
        notFound.push(name);
        console.log(`  ⚠ "${name}": não encontrada no mapeamento (mantida em ${subcat.stars} estrela)`);
      }
    });
  }

  // Atualizar timestamp
  data.last_updated = new Date().toISOString();

  // Salvar arquivo atualizado
  console.log('\n💾 Salvando alterações...');
  fs.writeFileSync(jsonPath, JSON.stringify(data, null, 2), 'utf8');

  console.log('\n✅ Atualização concluída!');
  console.log(`\n📊 Resumo:`);
  console.log(`  - Subcategorias atualizadas: ${updatedCount}`);
  console.log(`  - Subcategorias não encontradas: ${notFoundCount}`);

  if (notFound.length > 0) {
    console.log(`\n⚠️  Subcategorias não mapeadas:`);
    notFound.forEach(name => console.log(`    - ${name}`));
  }

  console.log(`\n⏰ Última atualização: ${data.last_updated}`);
}

// Executar
if (require.main === module) {
  try {
    updateStarsInJSON();
    process.exit(0);
  } catch (error) {
    console.error('\n❌ Erro:', error.message);
    process.exit(1);
  }
}

module.exports = { SUBCATEGORY_STARS, updateStarsInJSON };
