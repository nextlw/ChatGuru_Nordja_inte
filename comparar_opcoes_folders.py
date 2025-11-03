import json

# Lista fornecida pelo usuário (copie e cole aqui ou leia de um arquivo)
opcoes_menu = [
"Adriana Correia",
"Adriana Tavares",
"Agencia Fluida",
"AL Digital (Aline Bak)",
"Alejandra Aguilera (10)",
"Alessandra Caiado (20)",
"Alex e Yoanna",
"Aline Hochman",
"Amadeu e Rebeka (10)",
"Amure Pinho (10) Nova Fase",
"Amure Pinho (15)",
"Ana Flávia Carrilo (10)",
"Ana Flávia Tavares",
"Ana Paula Toledo",
"Anabela Cunha (5)",
"Anarella (Avulso)",
"Andrew Reider (10)",
"Antonio Vieira (Gostei)",
"Arcélia e Jorel Guilloty (14)",
"Arnaldo/Claudia",
"Atendimento ao Cliente",
"Atividades",
"Backline",
"Bianca Orsini (10)",
"Bruno Assis (10)",
"Bruno Sapienza (5)",
"Camila Mansano (5)",
"Carlos Secron (5)",
"Carolina Rosa (10)",
"Carolina Tavares (10)",
"Cassio Kienen (3 desejos)",
"Célio Assistência Arbel",
"Claudia Leicand (8)",
"Claudio e Gabriela",
"Cleivson Arruda (10)",
"Comece por aqui",
"Comunicados & Informações",
"Consuelo Maia (avulso)",
"Dadá Ribeiro (5)",
"Dani Hayoshi (10)",
"Daniel Rodrigues",
"Daniela Sitzer (8)",
"Danilo Buzar (3 desejos)",
"Darlann Costa (10)",
"David Politanski",
"Edwilson / Thaina (10)",
"Emylia Chamorro (10)",
"Fabio Tiepolo (8)",
"Fabricio Ramos (10)",
"Fernanda Cruz (5)",
"Fernanda Munhoz",
"Gabriel Benarros",
"Gabriela Avian (avulso)",
"Guilherme Lopes (5)",
"Gurgel D'Alfonso (Caio)",
"Henrique Caldeira (10)",
"Hernandes Benevides (8)",
"Humberto Siuves (10)",
"Hurá Bittencourt (10)",
"Ian Gallina (10)",
"Iara Biderman (8)",
"Igor Marchesini",
"Ille Cosmetics",
"Isabella Vasconcellos (14)",
"Jack Sarvary (10)",
"Jessica Joana Coral (10)",
"José Muritiba (10)",
"Julia Cambiaghi e Guilherme Lopes (10)",
"Kiko Augusto (Rei do Pitaco)",
"Laynara Coelho (10)",
"Leo Laranjeira (10)",
"Leticia e Breno",
"Lilian Palacios (10)",
"Lucas Riani (10)",
"Luciana Stracieri (10)",
"Luft Shoes",
"Marcelo Caldas (5)",
"Marco Bissi (10)",
"Margareth Darezzo",
"Mariana e David (10)",
"Mariana Paixão",
"Nino Vashakidze",
"Nordja",
"NSA Global (10)",
"Rafael Marcondes (Rei do Pitaco)",
"Rodolfo e Claudia",
"Thiago Martins (Rei do Pitaco)",
"William Duarte"
]

# Carregar nomes_folders.json
with open("nomes_folders.json", "r", encoding="utf-8") as f:
    nomes_folders = json.load(f)

set_menu = set(opcoes_menu)
set_folders = set(nomes_folders)

so_no_menu = sorted(set_menu - set_folders)
so_no_folders = sorted(set_folders - set_menu)
iguais = sorted(set_menu & set_folders)

print("Itens só no menu fornecido e NÃO no nomes_folders.json:")
for nome in so_no_menu:
    print("-", nome)

print("\nItens só no nomes_folders.json e NÃO no menu fornecido:")
for nome in so_no_folders:
    print("-", nome)

print(f"\nTotal de nomes idênticos nas duas listas: {len(iguais)}")
print(f"Total apenas no menu: {len(so_no_menu)}")
print(f"Total apenas no nomes_folders.json: {len(so_no_folders)}")