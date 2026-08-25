# mejorante

El hermano de [replicante](https://github.com/PascualMacana/replicante). Aquel **se copia**. Este **se copia y se mejora**.

No es un LLM. No ajusta pesos. No se propaga por la red. Es una Darwin Gödel Machine de juguete: el programa lleva su genoma embebido, muta un pedazo de sí mismo (el *cerebro*), lo mide contra un evaluador **congelado**, y si mejoró escribe un hijo que nace con esa versión.

```
padre  cerebro "0"           sse 2860
  │ evolve
  ▼
hijo   cerebro "(+ (* x x) (+ (* 3 x) 5))"   sse 0
```

## Qué es (y qué no)

| Concepto | Esto |
|---|---|
| **Replicante** | Copia el genoma. El hijo es igual, con otra generación. |
| **Máquina de Gödel** | Se reescribe solo si *demuestra* que el cambio es mejor. Teórica. |
| **Darwin Gödel Machine** | Se reescribe, y en vez de una prueba usa un benchmark empírico. Esto, en miniatura. |
| **Reina Roja** | El evaluador también evoluciona. Acá el evaluador está congelado a propósito. |
| **Gusano** | Se copia solo, a otros discos o redes. Esto **no**. |

El *cerebro* es una expresión en notación prefija (`(+ (* x x) 1)`). El evaluador pide aproximar

\[
f(x) = x^2 + 3x + 5
\]

en \(x = -5,\ldots,5\). El fitness es el error cuadrático (sse). Más bajo = mejor. Cero = óptimo.

El cerebro vive en una constante del propio `src/main.rs`. Mutarlo es reescribir código propio. El hijo, al compilarse, embebe esa fuente nueva: la mejora se hereda.

## Uso

```bash
cargo build --release
./target/release/mejorante identity
./target/release/mejorante evolve --steps 120 --spawn ./hijo --build
./hijo/target/debug/mejorante identity
```

Comandos:

```
mejorante identity              generación, linaje, cerebro, sse
mejorante eval [x]              cerebro vs objetivo
mejorante evolve                busca un cerebro mejor
                 --steps N      generaciones de búsqueda (default 120)
                 --lambda L     mutantes por paso (default 30)
                 --seed S       rng reproducible
                 --spawn <dir>  escribe el hijo con el campeón
                 --build        compila al hijo
                 --force        pisa un hijo anterior
                 --write        pisa el cerebro en este src/main.rs
mejorante spawn <dir>           copia el genoma actual (sin buscar)
mejorante genome                imprime las fuentes embebidas
```

`evolve --spawn` es Darwin: el padre no se toca, el hijo nace seleccionado.  
`evolve --write` es Lamarck: muta el fuente del proyecto actual; hay que recompilar para que el binario nazca de nuevo.

## Cómo funciona

1. Parsea `BRAIN` a un árbol (`x`, enteros, `+ - *`).
2. Cada paso genera λ mutantes (cambiar hoja, operador, crecer, achicar).
3. Se queda con el de menor `sse + 0.01·tamaño`.
4. Si hay `--spawn`, escribe un crate hijo con el `BRAIN` nuevo, `GENERATION+1` y el linaje.

El evaluador **no** se muta. Por eso no es Reina Roja: el criterio de "mejor" no se mueve. El programa puede mejorar hasta el óptimo y ahí se queda. Eso es honesto.

## Límites a propósito

- Un solo hijo por corrida. No hay loop, cron, ni red.
- No toca `$HOME`, `/`, `/usr`, `/etc`, ni el directorio actual.
- `--force` solo borra un destino que ya parece un `mejorante`.
- El espacio de búsqueda es un lenguaje chico, no Rust arbitrario. No reescribe el mutador ni el evaluador.
- No inventa un modelo nuevo. El "deep learning" de acá es un árbol de `+` y `*`.

## Por qué existe

[replicante](https://github.com/PascualMacana/replicante) sirve para tener en las manos la auto-replicación. Este sirve para la auto-mejora: el hijo no solo nació, nació **mejor**.
