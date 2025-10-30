use strsim::jaro_winkler;

fn normalize_client_name(name: &str) -> String {
    use deunicode::deunicode;

    deunicode(name)  // Remove acentos primeiro
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_numeric() && *c != '(' && *c != ')')
        .collect::<String>()
        .split_whitespace()  // Remove espaços extras
        .collect::<Vec<&str>>()
        .join(" ")
}

fn main() {
    const FUZZY_THRESHOLD: f64 = 0.85;
    
    // Cliente solicitante no ClickUp
    let clickup_client = "Hugo / NSA Global";
    
    // Possíveis entradas do ChatGuru
    let test_inputs = vec![
        "Hugo Tisaka",
        "NSA Global", 
        "Hugo",
        "Tisaka",
        "Hugo NSA Global",
        "NSA",
        "Hugo / NSA",
        "Hugo Tisaka NSA Global"
    ];
    
    println!("=== TESTE DE FUZZY MATCHING ===");
    println!("Cliente no ClickUp: '{}'", clickup_client);
    println!("Normalizado: '{}'", normalize_client_name(clickup_client));
    println!("Threshold: {:.0}%", FUZZY_THRESHOLD * 100.0);
    println!();
    
    for input in test_inputs {
        let normalized_input = normalize_client_name(input);
        let normalized_clickup = normalize_client_name(clickup_client);
        let similarity = jaro_winkler(&normalized_input, &normalized_clickup);
        
        let match_status = if similarity >= FUZZY_THRESHOLD {
            "✅ MATCH"
        } else {
            "❌ NO MATCH"
        };
        
        println!("Input: '{}' → Normalizado: '{}'", input, normalized_input);
        println!("Similaridade: {:.1}% {}", similarity * 100.0, match_status);
        println!("---");
    }
}
