## Como rodar

### Instalação e execução

```bash
git clone https://github.com/warubert/crablo.git
cd crablo
cargo run
```

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
