## Como rodar

### Instalação e execução

```bash
git clone https://github.com/warubert/crablo.git
cd crablo
cargo run
```

## Estrutura do código

```
src/
├── main.rs       # Entry point, AppState e loop principal
├── game.rs       # Struct Game — estado, lógica de update e draw
├── map.rs        # Tile, constantes, coordenadas isométricas, BFS e draw_walls
├── entities.rs   # Structs Monster e DmgText
└── renderer.rs   # draw_stickman (herói e monstros)
```

### Principais componentes

- **`Game`** (`game.rs`) — estado central do jogo (mapa, jogador, monstros, ouro, score)
- **`bfs()`** (`map.rs`) — algoritmo de pathfinding BFS usado pelo jogador e pelos monstros
- **`to_screen()` / `to_tile()`** (`map.rs`) — conversão entre coordenadas de tile e coordenadas de tela (projeção isométrica)
- **`draw_stickman()`** (`renderer.rs`) — renderiza o herói e os monstros com linhas simples
- **`draw_walls()`** (`map.rs`) — renderiza tiles de parede com faces 3D usando triângulos
- **`DmgText`** (`entities.rs`) — textos animados flutuantes para feedback visual de dano e ouro

## Estados do jogo

```
Menu → Playing → GameOver → Menu
```
