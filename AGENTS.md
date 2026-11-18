# Instruções para agentes

## Projeto e tecnologias

- Jogo 2D isométrico em Rust 2024 com Bevy 0.20.1; consulte [README.md](README.md) para execução, controles e visão geral dos arquivos.
- A janela usa o backend X11. Em `Cargo.toml`, `default-features = false` e a seleção explícita de features são intencionais. Preserve essa configuração e valide requisitos de plataforma antes de habilitar features agregadas do Bevy.
- O mapa atual tem 20 × 20 tiles. `Mapa1` implementa navegação BFS; `Warrior` implementa movimento temporizado; `Goblin` implementa decisões de movimento, ataque e passagem.
- A fonte Lato em `assets/fonts/Lato-Regular.ttf` dá suporte aos acentos do português. Preserve a licença em `assets/fonts/OFL.txt` e use essa fonte para novos textos de interface.

## Arquitetura e limites entre módulos

- `src/main.rs` configura o app Bevy, estados da aplicação, sistemas e integra entrada, interface e jogo.
- `src/game.rs` coordena turnos, combate, movimento e coleta. `Game` é genérico sobre `GameMap`, `PlayerActions` e `MonsterBehavior`; use essas traits ao adicionar implementações substituíveis.
- As interfaces de domínio ficam em `src/mapas/interface.rs`, `src/personagens/interface.rs` e `src/monstros/interface.rs`; implementações concretas ficam nas respectivas subpastas.
- `src/ui/window.rs` contém componentes e construtores reutilizáveis de painéis e botões. Mantenha esse módulo independente das regras e opções específicas do jogo.
- `src/ui/action_menu.rs` contém o estado e as opções específicas do menu de ações. A camada de aplicação traduz teclado e mouse em comandos do jogo.
- `src/renderer.rs` constrói a malha do mapa e desenha personagens, moedas e prévias de caminho. `src/mapas/geometry.rs` centraliza conversões e geometria isométrica.
- Não acople `Game` a componentes Bevy de interface ou ao estado de um menu. Comandos de jogo devem usar `Position`; a interface decide quando e como os enviar.
- Reutilize os construtores de janela e botão em novos contextos. Evite hitboxes com coordenadas de tela codificadas manualmente ou lógica de jogo dentro dos componentes visuais.
- Prefira enums tipados para conjuntos finitos de ações. Adicione traits quando houver um contrato ou implementações substituíveis reais; não crie abstrações apenas para cumprir SOLID.

## Testes e validação

- Execute `cargo fmt --all -- --check` e `cargo test` após mudanças em Rust.
- Os testes ficam em `tests/unit/` e são vinculados aos módulos por `#[cfg(test)]` e `#[path = "..."]`. Registre novos arquivos de teste no módulo correspondente.
- `cargo test` não abre janela nem cobre interação/renderização visual; valide manualmente fluxos visuais quando a mudança afetar a interface.
- Use `cargo run` para executar o jogo em um ambiente com suporte gráfico.

## Convenções de manutenção

- Mantenha nomes de tipos, funções e módulos em inglês, seguindo o código existente; mantenha documentação e textos apresentados ao jogador em português.
- Atualize o README quando controles, comportamento visível ou estrutura principal do projeto mudar.
- Evite dependências novas quando as APIs já disponíveis no Bevy ou na biblioteca padrão forem suficientes.
