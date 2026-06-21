use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

// 🏛️ SÍMBOLOS MITOLÓGICOS GREGOS (Substitutos temáticos de glifos/emojis)
const SIMBOLO_PASTA: &str = "Ω ";     // Ômega para pastas e diretórios estruturais
const SIMBOLO_APP: &str = "Ξ ";       // Xi para binários, executáveis e aplicativos centrais
const SIMBOLO_HISTORICO: &str = "Δ "; // Delta para histórico de invocações e mudanças
const SIMBOLO_ARQUIVO: &str = "α ";   // Alfa para arquivos locais comuns (imagens, textos, etc.)

// 🧠 CONTROLE DE NAVEGAÇÃO DE TRÊS VIAS (Gerencia o foco das colunas via TAB)
#[derive(PartialEq, Eq, Clone, Copy)]
enum ColunaAtiva {
    DiretorioEsquerda,
    AplicativosMeio,
    PreviewDireita,
}

#[derive(Clone, Debug)]
struct AppLauncher {
    nome: String,
    exec: String,
    invocacoes: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut busca = String::new();
    let mut lista_apps = escanear_aplicativos();
    carregar_historico(&mut lista_apps);
    
    let mut apps_filtrados = lista_apps.clone();
    let mut estado_lista = ListState::default();
    if !apps_filtrados.is_empty() {
        estado_lista.select(Some(0));
    }

    // Controle de navegação de diretório na Coluna 1
    let mut pasta_atual = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut itens_diretorio = obter_arquivos_da_pasta(&pasta_atual);
    
    // Otimizado para estilo Yazi/MC: pula o topo de histórico fixo e foca direto nos arquivos locais
    let mut estado_diretorio = ListState::default();
    if !itens_diretorio.is_empty() {
        estado_diretorio.select(Some(4));
    }

    // Estado de rolagem interna opcional para o painel de preview da direita
    let mut estado_preview = ListState::default();
    estado_preview.select(Some(0));

    // Define qual coluna manda no teclado (Inicializa na Central de Aplicativos)
    let mut coluna_ativa = ColunaAtiva::AplicativosMeio;

    let mut indice_cor = 0;
    loop {
        terminal.draw(|f| {
            let chunks_principais = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), 
                    Constraint::Min(1),    
                    Constraint::Length(3), 
                ])
                .split(f.size());

            // Divisão horizontal das 3 colunas (Proporções no estilo Miller Columns)
            let colunas_miller = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25), // Coluna 1: Histórico + Arquivos Locais
                    Constraint::Percentage(45), // Coluna 2: Lista Central de Aplicativos
                    Constraint::Percentage(30), // Coluna 3: Preview Híbrido Contextual
                ])
                .split(chunks_principais[1]);

            let (cor_destaque, cor_secundaria) = match indice_cor {
                1 => (Color::Rgb(212, 175, 55), Color::Rgb(100, 180, 255)),
                2 => (Color::Rgb(255, 34, 34), Color::Rgb(140, 190, 140)),
                3 => (Color::Rgb(255, 102, 0), Color::Rgb(0, 220, 220)),
                _ => (Color::Rgb(0, 255, 0), Color::Rgb(180, 120, 255)),
            };

            // Campo de Busca Superior
            let bloco_busca = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(cor_secundaria))
                .title(" Δ ORÁCULO DE DELFOS : INJECT_OPERATIONAL_VECTOR_NAME ");
            let campo_busca = Paragraph::new(busca.as_str())
                .block(bloco_busca)
                .style(Style::default().fg(cor_secundaria));
            f.render_widget(campo_busca, chunks_principais[0]);

            // 🧠 COLUNA 1: HISTÓRICO RÁPIDO + LISTA DE DIRETÓRIO (Esquerda)
            let mut itens_coluna1 = Vec::new();
            let historico_top3: Vec<String> = lista_apps
                .iter()
                .filter(|app| app.invocacoes > 0)
                .take(3)
                .map(|app| format!(" {} {} ({})", SIMBOLO_HISTORICO, app.nome, app.invocacoes))
                .collect();
                
            for linha_hist in historico_top3 {
                itens_coluna1.push(ListItem::new(linha_hist).style(Style::default().fg(cor_destaque)));
            }
            itens_coluna1.push(ListItem::new(" ───────────────────").style(Style::default().fg(cor_secundaria)));
            for item in &itens_diretorio {
                itens_coluna1.push(ListItem::new(item.clone()).style(Style::default().fg(cor_secundaria)));
            }

            // A borda esquerda acende se o foco estiver nela
            let estilo_borda_esq = if coluna_ativa == ColunaAtiva::DiretorioEsquerda {
                Style::default().fg(cor_destaque)
            } else {
                Style::default().fg(cor_secundaria)
            };

            let bloco_esquerdo = Block::default()
                .borders(Borders::ALL)
                .border_style(estilo_borda_esq)
                .title(" ⏳ MEMÓRIA / DIRETÓRIO ");
                
            // Convertido para stateful para gerenciar a navegação interna corretamente
            let widget_esquerdo = List::new(itens_coluna1)
                .block(bloco_esquerdo)
                .highlight_style(
                    Style::default()
                        .bg(Color::Rgb(20, 20, 20))
                        .fg(cor_destaque)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(" ➔ ");
            f.render_stateful_widget(widget_esquerdo, colunas_miller[0], &mut estado_diretorio);

            // 🧠 COLUNA 2: LISTA CENTRAL DE APLICATIVOS (Meio)
            let itens_central: Vec<ListItem> = apps_filtrados
                .iter()
                .map(|app| {
                    let prefixo = if app.invocacoes > 0 { SIMBOLO_HISTORICO } else { SIMBOLO_APP };
                    ListItem::new(format!("  {}  {}", prefixo, app.nome))
                        .style(Style::default().fg(cor_secundaria))
                })
                .collect();

            // A borda do meio acende se o foco estiver nela
            let estilo_borda_meio = if coluna_ativa == ColunaAtiva::AplicativosMeio {
                Style::default().fg(cor_destaque)
            } else {
                Style::default().fg(cor_secundaria)
            };

            let bloco_lista = Block::default()
                .borders(Borders::ALL)
                .border_style(estilo_borda_meio)
                .title(" Δ  CENTRAL DE VETORES ");

            let widget_lista = List::new(itens_central)
                .block(bloco_lista)
                .highlight_style(
                    Style::default()
                        .bg(Color::Rgb(20, 20, 20))
                        .fg(cor_destaque)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(" ➔ ");
            f.render_stateful_widget(widget_lista, colunas_miller[1], &mut estado_lista);

                       // 🧠 COLUNA 3: PAINEL DE PREVIEW HÍBRIDO EM ÁRVORE (Direita)
            let mut itens_preview_dir = Vec::new();
            let mut texto_preview_estatico = String::new();
            let mut mostrar_como_arvore = false;

            if coluna_ativa == ColunaAtiva::DiretorioEsquerda {
                // Se o usuário estiver navegando nos arquivos da esquerda, o painel direito vira a árvore/preview
                if let Some(i) = estado_diretorio.selected() {
                    if i >= 4 && (i - 4) < itens_diretorio.len() {
                        let item_real = &itens_diretorio[i - 4];
                        
                        // Limpa os símbolos para achar o caminho real
                        let nome_puro = item_real.trim()
                            .replace(SIMBOLO_PASTA, "")
                            .replace(SIMBOLO_ARQUIVO, "")
                            .replace(SIMBOLO_HISTORICO, "")
                            .replace(SIMBOLO_APP, "");
                            
                        let caminho_completo = pasta_atual.join(&nome_puro);

                        if caminho_completo.is_dir() {
                            // Se for uma pasta, lê os arquivos de dentro dela para mostrar a Tree
                            mostrar_como_arvore = true;
                            itens_preview_dir.push(ListItem::new(format!(" 🏛️ Conteúdo de: {}/", nome_puro)).style(Style::default().fg(cor_destaque)));
                            itens_preview_dir.push(ListItem::new(" ───────────────────").style(Style::default().fg(cor_secundaria)));
                            
                            if let Ok(sub_entradas) = fs::read_dir(&caminho_completo) {
                                for sub_entrada in sub_entradas.flatten() {
                                    let sub_path = sub_entrada.path();
                                    let sub_nome = sub_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                    let prefixo = if sub_path.is_dir() { SIMBOLO_PASTA } else { SIMBOLO_ARQUIVO };
                                    itens_preview_dir.push(ListItem::new(format!("   ├── {} {}", prefixo, sub_nome)).style(Style::default().fg(cor_secundaria)));
                                }
                            }
                            if itens_preview_dir.len() == 2 {
                                itens_preview_dir.push(ListItem::new("   └── (Pasta Vazia)").style(Style::default().fg(Color::DarkGray)));
                            }
                        } else {
                            // Se for um arquivo comum (como o main.rs), mostra os detalhes dele ou linhas iniciais
                            texto_preview_estatico = format!(
                                "\n  α ARQUIVO LOCAL\n\n  • Nome: {}\n  • Caminho: {}\n  • Tamanho: {} bytes\n\n  ───────────────────\n  [ENTER] Abrir via Nano/Nativo",
                                nome_puro,
                                caminho_completo.to_string_lossy(),
                                fs::metadata(&caminho_completo).map(|m| m.len()).unwrap_or(0)
                            );
                        }
                    }
                }
            } else {
               // Comportamento original: Se o foco estiver no meio, o painel direito exibe os metadados do aplicativo central
if let Some(i) = estado_lista.selected() {
    if i < apps_filtrados.len() {
        let selecionado = &apps_filtrados[i];
        texto_preview_estatico = format!(
            "\n  Δ VETOR OPERACIONAL\n\n  • Nome: {}\n  • Executável: {}\n  • Invocações: {}\n\n  ───────────────────\n  [ENTER] Executar via Fork\n  [F1] Editar via Nano\n  [F2] Imagem via Feh/Sww\n  [F3] Navegar via Yazi",
            selecionado.nome, selecionado.exec, selecionado.invocacoes
        );
    }
}

            }

            let estilo_borda_dir = if coluna_ativa == ColunaAtiva::PreviewDireita {
                Style::default().fg(cor_destaque)
            } else {
                Style::default().fg(cor_secundaria)
            };

            let bloco_preview = Block::default()
                .borders(Borders::ALL)
                .border_style(estilo_borda_dir)
                .title(" Δ  PREVIEW DO VETOR ");

            // Renderiza como Lista se for a árvore de diretórios, ou como Parágrafo se for texto de metadados
            if mostrar_como_arvore {
                let widget_preview_tree = List::new(itens_preview_dir).block(bloco_preview);
                f.render_widget(widget_preview_tree, colunas_miller[2]);
            } else {
                let widget_preview = Paragraph::new(texto_preview_estatico.as_str())
                    .block(bloco_preview)
                    .style(Style::default().fg(cor_secundaria))
                    .wrap(Wrap { trim: false });
                f.render_widget(widget_preview, colunas_miller[2]);
            }

                    // Barra de Atalhos no Rodapé com o indicador da Tecla H para a Home
let rodape_texto = Line::from(vec![
    Span::styled(" h ", Style::default().bg(cor_destaque).fg(Color::Black).add_modifier(Modifier::BOLD)),
    Span::styled(" Home ❖ ", Style::default().fg(cor_secundaria)),
    Span::styled(" F1 ", Style::default().bg(cor_destaque).fg(Color::Black).add_modifier(Modifier::BOLD)),
    Span::styled(" Nano ❖ ", Style::default().fg(cor_secundaria)),
    Span::styled(" F2 ", Style::default().bg(cor_destaque).fg(Color::Black).add_modifier(Modifier::BOLD)),
    Span::styled(" Fastfetch ❖ ", Style::default().fg(cor_secundaria)),
    Span::styled(" F3 ", Style::default().bg(cor_destaque).fg(Color::Black).add_modifier(Modifier::BOLD)),
    Span::styled(" Yazi ❖ ", Style::default().fg(cor_secundaria)),
    Span::styled(" F4 ", Style::default().bg(cor_destaque).fg(Color::Black).add_modifier(Modifier::BOLD)),
    Span::styled(" Espectro ❖ ", Style::default().fg(cor_secundaria)),
    Span::styled(" F5 ", Style::default().bg(cor_destaque).fg(Color::Black).add_modifier(Modifier::BOLD)),
    Span::styled(" wallpaper ❖ ", Style::default().fg(cor_secundaria)),
    Span::styled(" F10 ", Style::default().bg(Color::Rgb(150, 0, 0)).fg(Color::White).add_modifier(Modifier::BOLD)),
    Span::styled(" TERMINATE ", Style::default().fg(cor_secundaria)),
]);



            let bloco_rodape = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(cor_secundaria))
                .title(" Δ ÉGIDE DE ATENAS : CORE_ORCHESTRATION_PANEL ");
            let widget_rodape = Paragraph::new(rodape_texto).block(bloco_rodape);
            f.render_widget(widget_rodape, chunks_principais[2]);
        })?;
        // -----------------------------------------------------------------
                // -----------------------------------------------------------------
        // CAPTURA DE EVENTOS DE TECLADO (MOTOR DINÂMICO HÍBRIDO COM CONTROLE DE FOCO)
        // -----------------------------------------------------------------
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                
                // 🧠 TECLA TAB: Alterna o foco sequencialmente entre as 3 colunas (Esquerda -> Meio -> Direita)
                KeyCode::Tab => {
                    coluna_ativa = match coluna_ativa {
                        ColunaAtiva::DiretorioEsquerda => ColunaAtiva::AplicativosMeio,
                        ColunaAtiva::AplicativosMeio => ColunaAtiva::PreviewDireita,
                        ColunaAtiva::PreviewDireita => ColunaAtiva::DiretorioEsquerda,
                    };
                }
                
                KeyCode::Up => {
                    match coluna_ativa {
                        ColunaAtiva::AplicativosMeio => {
                            if let Some(i) = estado_lista.selected() {
                                if i > 0 { estado_lista.select(Some(i - 1)); }
                            }
                        }
                        ColunaAtiva::DiretorioEsquerda => {
                            if let Some(i) = estado_diretorio.selected() {
                                if i > 4 { estado_diretorio.select(Some(i - 1)); }
                            }
                        }
                        ColunaAtiva::PreviewDireita => {
                            if let Some(i) = estado_preview.selected() {
                                if i > 0 { estado_preview.select(Some(i - 1)); }
                            }
                        }
                    }
                }
                
                KeyCode::Down => {
                    match coluna_ativa {
                        ColunaAtiva::AplicativosMeio => {
                            if let Some(i) = estado_lista.selected() {
                                if i < apps_filtrados.len() - 1 { estado_lista.select(Some(i + 1)); }
                            }
                        }
                        ColunaAtiva::DiretorioEsquerda => {
                            if let Some(i) = estado_diretorio.selected() {
                                if i < (itens_diretorio.len() + 4 - 1) { estado_diretorio.select(Some(i + 1)); }
                            }
                        }
                        ColunaAtiva::PreviewDireita => {
                            if let Some(i) = estado_preview.selected() {
                                if i < 15 { estado_preview.select(Some(i + 1)); }
                            }
                        }
                    }
                }

                KeyCode::Backspace => {
                    if coluna_ativa == ColunaAtiva::AplicativosMeio {
                        busca.pop();
                        apps_filtrados = filtrar_apps(&lista_apps, &busca);
                        if !apps_filtrados.is_empty() { estado_lista.select(Some(0)); }
                    }
                }

                // 🧠 O 'h' fica no topo absoluto de todas as capturas de letras!
                KeyCode::Char('h') if coluna_ativa != ColunaAtiva::AplicativosMeio => {
                    if let Some(caminho_home) = env::var_os("HOME") {
                        pasta_atual = PathBuf::from(caminho_home);
                        itens_diretorio = obter_arquivos_da_pasta(&pasta_atual);
                        estado_diretorio.select(Some(4));
                        coluna_ativa = ColunaAtiva::DiretorioEsquerda;
                    }
                }

                // 🧠 MATAR MÚSICAS: Pressionar a letra 'm' (Limpeza forçada via pkill de barramento)
                KeyCode::Char('m') if coluna_ativa != ColunaAtiva::AplicativosMeio => {
                    let _ = Command::new("pkill")
                        .arg("-f")
                        .arg("archonplayer")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
                
                // 🧠 Apenas UM Char(c) genérico para a busca
                KeyCode::Char(c) => {
                    if coluna_ativa == ColunaAtiva::AplicativosMeio {
                        busca.push(c);
                        apps_filtrados = filtrar_apps(&lista_apps, &busca);
                        if !apps_filtrados.is_empty() { estado_lista.select(Some(0)); }
                    }
                }
              
                // 🧠 Retorno de diretório
                KeyCode::Left => {
                    if coluna_ativa == ColunaAtiva::DiretorioEsquerda || coluna_ativa == ColunaAtiva::PreviewDireita {
                        if let Some(pai) = pasta_atual.parent() {
                            pasta_atual = pai.to_path_buf();
                            itens_diretorio = obter_arquivos_da_pasta(&pasta_atual);
                            estado_diretorio.select(Some(4)); 
                            coluna_ativa = ColunaAtiva::DiretorioEsquerda; 
                        }
                    }
                }

                                             KeyCode::Enter => {
                    if coluna_ativa == ColunaAtiva::AplicativosMeio {
                        if let Some(i) = estado_lista.selected() {
                            if i < apps_filtrados.len() {
                                let app_selecionado = apps_filtrados[i].clone();
                                registrar_uso_historico(&app_selecionado.nome, &mut lista_apps);
                                
                                // 🏛️ CENTRAL: Garante reset estável para os utilitários de terminal nativos
                                if app_selecionado.exec == "nano" || app_selecionado.exec == "yazi" {
                                    busca.clear();
                                    apps_filtrados = lista_apps.clone();
                                    restaurar_interface_terminal(&mut terminal)?;
                                    executar_via_double_fork(&app_selecionado.exec);
                                    enable_raw_mode()?;
                                    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                                    terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
                                    terminal.clear()?;
                                } else {
                                    executar_via_double_fork(&app_selecionado.exec);
                                    busca.clear();
                                    apps_filtrados = lista_apps.clone();
                                }
                                if !apps_filtrados.is_empty() { estado_lista.select(Some(0)); }
                            }
                        }
                    } else if coluna_ativa == ColunaAtiva::DiretorioEsquerda {
                        if let Some(i) = estado_diretorio.selected() {
                            if i >= 4 && (i - 4) < itens_diretorio.len() {
                                let item_real = &itens_diretorio[i - 4];
                                
                                // 🏛️ REPARADO: Remove os símbolos gregos e limpa espaços fantasmas residuais no final!
                                let nome_puro = item_real
                                    .replace(SIMBOLO_PASTA, "")
                                    .replace(SIMBOLO_ARQUIVO, "")
                                    .replace(SIMBOLO_HISTORICO, "")
                                    .replace(SIMBOLO_APP, "")
                                    .trim()
                                    .to_string();
                                    
                                let caminho_completo = pasta_atual.join(&nome_puro);
                                let caminho_str = caminho_completo.to_string_lossy();
                                let ext = caminho_completo.extension()
                                    .map(|e| e.to_string_lossy().to_lowercase())
                                    .unwrap_or_default();
                                
                                // 🧠 DETECÇÃO DE BARRAMENTO GRÁFICO (Atenas Kernel Detect)
                                let sessao_wayland = env::var("WAYLAND_DISPLAY").is_ok();
                             // 🏛️ 1. PROTOCOLO PARA IMAGENS UNIVERSAL COM FEH GARANTIDO (dwm e dwl)
                                if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "gif" {
                                    if sessao_wayland {
                            // 🔮 MODO DWL (Wayland): Aplica no fundo da tela usando swaybg/swww
                                        if Command::new("swww").arg("img").arg(&caminho_completo).status().is_err() {
                                            let _ = Command::new("swaybg")
                                                .args(&["-m", "fill", "-i", &caminho_str])
                                                .spawn();
                                        }
                                    } else {
                         // 🔮 MODO DWM (X11): O feh garante a aplicação do wallpaper no fundo imediatamente!
                                        let _ = Command::new("feh")
                                            .args(&["--bg-fill", &caminho_str]) // ➔ Ajustado exatamente para caminho_str
                                            .status();

                                        // Abre o nsxiv flutuante por cima via double fork para visualização isolada
                                        executar_via_double_fork(&format!("nsxiv -a -n 1 \"{}\"", caminho_str));
                                    }
                                }


                                // 🏛️ 2. PROTOCOLO PARA MÚSICA (ARCHONPLAYER COMPLETO EM PAINEL SEPARADO)
                                else if ext == "mp3" || ext == "wav" || ext == "flac" || ext == "ogg" {
                                    let _ = Command::new("pkill").args(&["-f", "archonplayer"]).status();
                                    
                                    if sessao_wayland {
                                        executar_via_double_fork(&format!("foot -e archonplayer \"{}\"", caminho_str));
                                    } else {
                                        executar_via_double_fork(&format!("qterminal -e archonplayer \"{}\"", caminho_str));
                                    }
                                } 
                                       // 🏛️ 3. PROTOCOLO PARA ARQUIVOS (NANO EM JANELA FLUTUANTE SEPARADA)
                       else if caminho_completo.is_file() {
    // Opcional: Se você tiver um protocolo de imagem (ex: .png, .jpg), pode ignorá-lo aqui:
                      let ext = caminho_completo.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();   
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
        // Se tiver lógica para imagens, ela entra aqui (ex: abrir com feh/swww)
                        } else {
        // Abre ABSOLUTAMENTE QUALQUER OUTRO ARQUIVO (como .bashrc, arquivos sem extensão, .conf, etc.)
                             if sessao_wayland {
                            executar_via_double_fork(&format!("foot -e nano \"{}\"", caminho_str));
                             } else {
                           executar_via_double_fork(&format!("qterminal -e nano \"{}\"", caminho_str));
                           }
                      }
                 } 
                  // 🏛️ 4. NAVEGAÇÃO DE DIRETÓRIOS
                     else if caminho_completo.is_dir() {
                    pasta_atual = caminho_completo;
                    itens_diretorio = obter_arquivos_da_pasta(&pasta_atual);
                     estado_diretorio.select(Some(4));
                               }
                            }
                        }
                    }
                } // Fecha o KeyCode::Enter

                KeyCode::F(1) => {
                    // 📝 ATALHO F1 ATUALIZADO: Agora abre o NANO interativo diretamente em tela cheia na TUI atual!
                    busca.clear();
                    apps_filtrados = lista_apps.clone();
                    restaurar_interface_terminal(&mut terminal)?;
                    executar_via_double_fork("nano");
                    enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                    terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
                    terminal.clear()?;
                }
                
                KeyCode::F(2) => {
                    busca.clear();
                    apps_filtrados = lista_apps.clone();
                    restaurar_interface_terminal(&mut terminal)?;
                    print!("\x1b[2J\x1b[1;1H");
                    let _ = Command::new("fastfetch").status();
                    println!("\nPressione Enter para voltar...");
                    let mut buffer = String::new();
                    let _ = io::stdin().read_line(&mut buffer);
                    enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                    terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
                    terminal.clear()?;
                }
                
                KeyCode::F(3) => {
                    busca.clear();
                    apps_filtrados = lista_apps.clone();
                    restaurar_interface_terminal(&mut terminal)?;
                    executar_via_double_fork("yazi");
                    enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                    terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
                    terminal.clear()?;
                }
                
                KeyCode::F(4) => { 
                    indice_cor = (indice_cor + 1) % 4;
                    let seq_ansi = match indice_cor {
                        1 => "\x1b]10;#d4af37\x07\x1b]11;#000000\x07",
                        2 => "\x1b]10;#ff2222\x07\x1b]11;#000000\x07",
                        3 => "\x1b]10;#ff6600\x07\x1b]11;#000000\x07",
                        _ => "\x1b]10;#00ff00\x07\x1b]11;#000000\x07",
                    };
                    print!("{}", seq_ansi);
                    io::stdout().flush().ok();
                }

                KeyCode::F(5) => {
                    restaurar_interface_terminal(&mut terminal)?;
                    gerenciador_wallpaper_universal();
                    enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                    terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
                    terminal.clear()?;
                }
                
                KeyCode::F(10) => break,
                _ => {}
            } // Fecha o match key.code
        } // Fecha o if let Event::Key
    } // Fecha o loop principal

    restaurar_interface_terminal(&mut terminal)?;
    Ok(())
} // Fecha a função main()

                          

fn restaurar_interface_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

// 🏛️ DOUBLE FORK EM BAIXO NÍVEL POSIX: Executa sem travar o loop de inputs do Rust
fn executar_via_double_fork(comando: &str) {
    let cmd = comando.to_string();
    let binario_base = cmd.split_whitespace().next().unwrap_or("");
           // 🧠 CRÍTICO: Mapeia quais aplicativos assumem o terminal em primeiro plano (Foreground)
    let é_app_de_terminal = binario_base == "nano" || binario_base == "yazi" || binario_base == "nano" || binario_base == "archonplayer";


    
    unsafe {
        if é_app_de_terminal {
            match libc::fork() {
                0 => {
                    print!("\x1b[2J\x1b[1;1H");
                    io::stdout().flush().ok();
                    let _ = Command::new("clear").status();

                    let partes: Vec<&str> = cmd.split_whitespace().collect();
                    let mut construtor = Command::new(partes[0]);
                    if partes.len() > 1 {
                        construtor.args(&partes[1..]);
                    }

                    construtor
                        .env_clear() 
                        .env("TERM", env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string()))
                        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit());

                    if let Ok(mut processo) = construtor.spawn() {
                        let _ = processo.wait();
                    }
                    
                    print!("\x1b[2J\x1b[1;1H");
                    io::stdout().flush().ok();
                    let _ = Command::new("clear").status();
                    libc::_exit(0);
                }
                p if p > 0 => {
                    libc::waitpid(p, std::ptr::null_mut(), 0);
                }
                _ => {}
            }
            return;
        }

        // Bifurcação dupla para desacoplar totalmente apps gráficos da TUI do Oráculo
        match libc::fork() {
            0 => {
                libc::setsid();
                match libc::fork() {
                    0 => {
                        Command::new("sh")
                            .arg("-c")
                            .arg(&cmd)
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                            .ok();
                        libc::_exit(0);
                    }
                    _ => libc::_exit(0),
                }
            }
            p if p > 0 => {
                libc::waitpid(p, std::ptr::null_mut(), 0);
            }
            _ => {}
        }
    }
}

fn escanear_aplicativos() -> Vec<AppLauncher> {
    let mut apps = Vec::new();
    let caminhos = ["/usr/share/applications", "/usr/local/share/applications"];
    
    for caminho in &caminhos {
        if let Ok(entradas) = fs::read_dir(caminho) {
            for entrada in entradas.flatten() {
                let path = entrada.path();
                if path.extension().map_or(false, |ext| ext == "desktop") {
                    if let Ok(conteudo) = fs::read_to_string(&path) {
                        let mut nome = String::new();
                        let mut exec = String::new();
                        for linha in conteudo.lines() {
                            if linha.starts_with("Name=") && nome.is_empty() {
                                nome = linha.replace("Name=", "");
                            }
                            if linha.starts_with("Exec=") && exec.is_empty() {
                                exec = linha.replace("Exec=", "").split_whitespace().next().unwrap_or("").to_string();
                            }
                        }
                        if !nome.is_empty() && !exec.is_empty() {
                            apps.push(AppLauncher { nome, exec, invocacoes: 0 });
                        }
                    }
                }
            }
        }
    }
    apps.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
    apps
}
// 🧠 ADAPTADO: Lista os arquivos substituindo os glifos antigos por símbolos do alfabeto grego
fn obter_arquivos_da_pasta(caminho: &Path) -> Vec<String> {
    let mut itens = Vec::new();
    if let Ok(entradas) = fs::read_dir(caminho) {
        for entrada in entradas.flatten() {
            let path = entrada.path();
            let nome_arquivo = path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| String::from("?"));
                
            if path.is_dir() {
                itens.push(format!(" {}{}/", SIMBOLO_PASTA, nome_arquivo));
            } else {
                let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                let icone = match ext.as_str() {
                    "png" | "jpg" | "jpeg" | "gif" => SIMBOLO_ARQUIVO,
                    "txt" | "md" | "sh" | "rs" | "toml" => SIMBOLO_ARQUIVO,
                    "mp3" | "wav" | "flac" => SIMBOLO_ARQUIVO,
                    _ => SIMBOLO_APP, 
                };
                itens.push(format!(" {}{}", icone, nome_arquivo));
            }
        }
    }
    itens.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    itens
}

fn filtrar_apps(apps: &[AppLauncher], busca: &str) -> Vec<AppLauncher> {
    if busca.is_empty() { return apps.to_vec(); }
    
    let busca_rebaixada = busca.to_lowercase();
    let mut filtrados: Vec<(AppLauncher, usize)> = Vec::new();

    for app in apps {
        let nome_rebaixado = app.nome.to_lowercase();
        let mut index_busca = 0;
        let mut match_encontrado = true;
        
        for char_busca in busca_rebaixada.chars() {
            if let Some(pos) = nome_rebaixado[index_busca..].find(char_busca) {
                index_busca += pos + char_busca.len_utf8();
            } else {
                match_encontrado = false;
                break;
            }
        }

        if match_encontrado {
            filtrados.push((app.clone(), index_busca));
        }
    }

    filtrados.sort_by(|a, b| {
        b.0.invocacoes.cmp(&a.0.invocacoes)
            .then_with(|| a.1.cmp(&b.1))
    });

    filtrados.into_iter().map(|(app, _)| app).collect()
}

fn carregar_historico(apps: &mut [AppLauncher]) {
    if let Some(caminho_home) = env::var_os("HOME") {
        let path_cache = Path::new(&caminho_home).join(".cache/archia_history");
        if let Ok(conteudo) = fs::read_to_string(path_cache) {
            for linha in conteudo.lines() {
                let partes: Vec<&str> = linha.split('|').collect();
                if partes.len() == 2 {
                    let nome_app = partes[0].to_string();
                    if let Ok(qtd) = partes[1].parse::<u32>() {
                        if let Some(app) = apps.iter_mut().find(|a| a.nome == nome_app) {
                            app.invocacoes = qtd;
                        }
                    }
                }
            }
        }
    }
    apps.sort_by(|a, b| b.invocacoes.cmp(&a.invocacoes).then_with(|| a.nome.to_lowercase().cmp(&b.nome.to_lowercase())));
}

fn registrar_uso_historico(nome_app: &str, apps: &mut Vec<AppLauncher>) {
    if let Some(app) = apps.iter_mut().find(|a| a.nome == nome_app) {
        app.invocacoes += 1;
    }
    
    apps.sort_by(|a, b| b.invocacoes.cmp(&a.invocacoes).then_with(|| a.nome.to_lowercase().cmp(&b.nome.to_lowercase())));

    if let Some(caminho_home) = env::var_os("HOME") {
        let path_cache = Path::new(&caminho_home).join(".cache/archia_history");
        if let Some(pai) = path_cache.parent() {
            let _ = fs::create_dir_all(pai);
        }
        if let Ok(mut arquivo) = fs::File::create(path_cache) {
            for app in apps {
                if app.invocacoes > 0 {
                    let _ = writeln!(arquivo, "{}|{}", app.nome, app.invocacoes);
                }
            }
        }
    }
}

fn gerenciador_wallpaper_universal() {
    println!("\x1b[1;30m╔═══════════════════ ⚡ CADINHO DE HEFESTO : WALLPAPER CORE ⚡ ═══════════════════╗\x1b[0m");
    println!("\x1b[1;30m║\x1b[0m   \x1b[1;31mINJECT NEW PLATFORM VISUAL CANVAS THROUGH DISPLAY OR WAYLAND INSTANCES\x1b[0m      \x1b[1;30m║\x1b[0m");
    println!("\x1b[1;30m╚═════════════════════════════════════════════════════════════════════════════════╝\x1b[0m\n");

    print!("\x1b[1;30m[HERMES]:\x1b[0m Arraste o arquivo ou insira o caminho completo do elemento gráfico:\n\x1b[1;31m>>> \x1b[0m");
    io::stdout().flush().ok();

    let mut caminho_img = String::new();
    io::stdin().read_line(&mut caminho_img).ok();
    let caminho_img = caminho_img.trim().replace('\'', "").replace('"', "");

    if !Path::new(&caminho_img).exists() {
        println!("\n\x1b[1;31m╔═════════════════════════════ 🏮 ERROR_CARONTE_GATE 🏮 ═════════════════════════╗\x1b[0m");
        println!("\x1b[1;31m║\x1b[0m  [-] Falha crítica na busca do elemento: Arquivo gráfico não localizado!       \x1b[1;31m║\x1b[0m");
        println!("\x1b[1;31m╚═════════════════════════════════════════════════════════════════════════════════╝\x1b[0m");
        println!("\nPressione Enter para abortar operação visual...");
        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
        return;
    }

    println!("\n\x1b[1;30m[+] Escaneando barramento gráfico (Atenas Kernel Detect)...\x1b[0m");
    let sessao_wayland = env::var("WAYLAND_DISPLAY").is_ok();
    let sessao_x11 = env::var("DISPLAY").is_ok();

    if sessao_wayland {
        println!("\x1b[1;32m[->] Barramento Wayland Detectado (dwl). Renderizando via swww/swaybg...\x1b[0m");
        if Command::new("swww").arg("img").arg(&caminho_img).status().is_err() {
            let _ = Command::new("swaybg")
                .args(&["-i", &caminho_img, "-m", "fill"])
                .spawn();
        }
    } else if sessao_x11 {
        println!("\x1b[1;32m[->] Barramento X11 Detectado (dwm/Fluxbox). Renderizando via feh...\x1b[0m");
        let _ = Command::new("feh")
            .args(&["--bg-fill", &caminho_img])
            .status();

        println!("[+] Estabilizando compositor de transparências xcompmgr...");
        let _ = Command::new("killall").arg("xcompmgr").status();
        std::thread::sleep(std::time::Duration::from_millis(600)); 
        let _ = Command::new("xcompmgr")
            .args(&["-c", "-C", "-t-5", "-l-5", "-r4.2", "-o.55"])
            .spawn();
    } else {
        println!("\x1b[1;31m[!] Caos crítico: Servidor gráfico ausente ou inacessível.\x1b[0m");
    }

    println!("\nPressione Enter para retornar ao painel central...");
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
}
