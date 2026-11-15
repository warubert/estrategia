## Como rodar

### Instalação e execução

O jogo usa a engine Bevy 0.20.1 e requer Rust 1.97.1 ou mais recente.

```bash
git clone https://github.com/warubert/crablo.git
cd crablo
cargo run
```

### Testes automatizados

Execute `cargo test` para verificar as regras de turnos, combate, coleta de
moedas, movimento, navegação e coordenadas isométricas. Esses testes não abrem
uma janela e não verificam a renderização ou os cliques reais do mouse.
Os testes ficam em `tests/unit/`, separados por arquivo de código testado e
vinculados como testes unitários aos seus módulos.

## Como jogar

O jogo é por turnos: o jogador age primeiro, depois cada monstro vivo age uma
vez, e então começa uma nova rodada. No turno do jogador, passe o mouse sobre
um destino para visualizar o caminho e clique para percorrê-lo até o destino
como uma única ação. Os monstros agem após o movimento terminar. O caminho
contorna monstros; ao clicar em um monstro distante, o jogador avança até ficar
adjacente a ele. Em outro turno, clique no monstro adjacente para atacá-lo em vez
de se mover.

Cada monstro ataca se estiver adjacente ao jogador; caso contrário, tenta
avançar um tile. Um monstro sem caminho livre passa sua vez. Cliques em paredes,
fora do mapa ou em destinos inacessíveis não consomem o turno. As moedas são
coletadas ao entrar no tile correspondente. A partida termina quando o jogador
morre ou todos os monstros são derrotados.

## Estrutura do código

```
src/
├── main.rs       # App Bevy, recursos do jogo, entrada, HUD e estados
├── game.rs       # Orquestra turnos, combate, moedas e interface
├── personagens/
│   ├── mod.rs        # Exportações do domínio
│   ├── interface.rs  # Trait PlayerActions e resultado do movimento
│   └── warrior/
│       └── mod.rs    # Implementação do jogador Warrior
├── monstros/
│   ├── mod.rs        # Exportações do domínio
│   ├── interface.rs  # Trait MonsterBehavior e ações
│   └── goblin/
│       └── mod.rs    # Implementação do monstro Goblin
├── mapas/
│   ├── mod.rs        # Exportações e constantes compartilhadas
│   ├── interface.rs  # Trait GameMap, posição e tile
│   ├── geometry.rs   # Coordenadas isométricas e distância
│   └── mapa1/
│       └── mod.rs    # Implementação do Mapa1 e BFS
├── entities.rs   # Texto flutuante de dano e ouro
└── renderer.rs   # Malha 2D Bevy para mapa, caminho e personagens
tests/
└── unit/
    ├── game.rs       # Turnos e regras do jogo
    ├── geometry.rs   # Coordenadas isométricas
    ├── mapa1.rs      # Navegação e tiles do mapa
    ├── goblin.rs     # Comportamento do monstro
    └── warrior.rs    # Estado e movimento do jogador
```

### Principais componentes

- **`Game`** (`game.rs`) — coordena os componentes e a ordem dos turnos, usando tipos genéricos para os três domínios
- **`PlayerActions` / `Warrior`** (`personagens/`) — interface e implementação do estado e movimento do jogador
- **`MonsterBehavior` / `Goblin`** (`monstros/`) — interface e implementação das decisões dos monstros
- **`GameMap` / `Mapa1`** (`mapas/`) — acesso aos tiles e cálculo de caminhos por BFS; jogador e monstros dependem da interface
- **`to_screen()` / `to_tile()`** (`mapas/geometry.rs`) — conversão entre coordenadas de tile e coordenadas de tela (projeção isométrica)
- **`GameSession`** (`main.rs`) — conecta entrada do Bevy às regras independentes do jogo
- **`renderer.rs`** — constrói a malha 2D isométrica e atualiza hover/caminho
- **`DmgText`** (`entities.rs`) — textos animados flutuantes para feedback visual de dano e ouro

Para acrescentar uma implementação, crie uma subpasta na pasta do domínio,
implemente a trait de `interface.rs` e declare/exporte o módulo em `mod.rs`.
As implementações atuais são selecionadas na construção de `Game`. Atualmente
uma partida usa um único tipo concreto de monstro por vez.

## Estados do jogo

```
Menu → Playing → GameOver → Menu
```
