# IraGpt - Team Balancer

Este projeto é uma ferramenta em Rust desenvolvida para equilibrar times de futebol (ou qualquer esporte) de forma otimizada. Ele utiliza **Programação Linear** para garantir que as características dos jogadores (como ataque, defesa, velocidade, etc.) sejam distribuídas da maneira mais justa possível entre as equipes.

## O que é Programação Linear?

A Programação Linear (PL) é uma técnica matemática usada para encontrar o melhor resultado possível (como o menor custo ou o maior lucro) em um modelo matemático cujos requisitos são representados por relações lineares. 

Neste projeto, usamos um subconjunto da PL chamado **Programação Linear Inteira Mista (MILP)** para decidir se um jogador deve ou não estar em um determinado time. Como não podemos dividir um jogador ao meio (ele está no Time A *ou* no Time B), usamos variáveis binárias (0 ou 1).

## Fórmulas Matemáticas Usadas

O modelo busca minimizar a diferença de qualidade entre os times. Para cada par (Jogador $i$, Time $j$), definimos:

$`x_{i,j} \in \{0, 1\}`$ (Variável binária: 1 se o jogador $i$ está no time $j$, 0 caso contrário).

### 1. Função Objetivo
O objetivo é minimizar a soma das diferenças máximas de cada atributo entre os times e a média global:

$`\text{Minimizar } \sum_{k \in \text{Critérios}} \text{max\_diff}_k`$

Onde $`\text{max\_diff}_k`$ é uma variável que captura o maior desvio entre a nota de um time em um critério $k$ e a média esperada para aquele critério.

### 2. Restrições
*   **Um jogador por time:** Cada jogador selecionado deve ser alocado em exatamente um time.
    $$\sum_{j=1}^{T} x_{i,j} = 1, \forall i$$
*   **Tamanho do time:** Cada time deve ter exatamente o número definido de jogadores (ex: 5).
    $$\sum_{i=1}^{P} x_{i,j} = N, \forall j$$
*   **Equilíbrio de Atributos:** Para cada critério $k$ (Goleiro, Zagueiro, Meio, etc.), a diferença entre a nota do time e a média global deve ser menor ou igual a $`\text{max\_diff}_k`$.

## Como fazer o Build

Este projeto utiliza o solver **HiGHS** para resolver o problema de otimização.

### Instalação do HiGHS
Antes de compilar o projeto em Rust, você deve ter o HiGHS instalado no seu sistema. 

**Passos para macOS (via Source):**
1. Certifique-se de ter o `CMake` instalado (`brew install cmake`).
2. Clone o repositório oficial:
   ```bash
   git clone https://github.com/ERGO-Code/HiGHS.git
   cd HiGHS
   mkdir build
   cd build
   cmake ..
   make
   sudo make install
   ```

Para instruções detalhadas de outras plataformas ou métodos de instalação, consulte a [documentação oficial do HiGHS](https://ergo-code.github.io/HiGHS/dev/installation/).

### Compilando o Projeto
Com o HiGHS instalado, basta rodar o comando padrão do Cargo:

```bash
cargo build --release
```

## Como Usar

1. Prepare um arquivo `players.json` com a lista de jogadores e suas notas.
2. Execute o binário:
   ```bash
   ./target/release/IraGpt
   ```
3. Selecione os jogadores disponíveis no dia através da interface interativa.
4. O programa exibirá a composição dos times balanceados.
