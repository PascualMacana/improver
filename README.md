# mejorante

Un programa chico en Rust que reescribe un pedazo de su propia fuente, comprueba si el cambio es mejor, y puede escribir un proyecto hijo que hereda la mejora.

No es un modelo de lenguaje y no se copia solo. Le señalás una carpeta; sólo escribe ahí.

Lo que evoluciona es una expresión matemática chica, el **cerebro**. El programa intenta encajar esta función:

```
f(x) = x² + 3x + 5
```

en `x = -5 … 5`. El puntaje es la suma de errores al cuadrado (**sse**). Más bajo es mejor. Cero es un encaje perfecto.

```
padre     cerebro  0                              sse  4323
  │ evolve
  ▼
hijo      cerebro  (+ (+ (* 3 x) (* x x)) 5)      sse  0
```

Esa expresión hija es `x² + 3x + 5`. La búsqueda puede encontrar primero un equivalente más largo; después las identidades algebraicas lo achican.

![Una célula que se llena a medida que encaja la curva](cell.svg)

Míralo en la terminal. Una célula que se va llenando mientras baja el error. A la derecha: la curva objetivo vs el cerebro, y cómo cae el error a lo largo de la búsqueda. `dish` usa por defecto la seed 7, que llega al encaje perfecto en pocos pasos.

```bash
cargo run -- dish
```

## Cómo correrlo

Hace falta [Rust](https://rustup.rs/).

```bash
cargo build --release
./target/release/mejorante identity
./target/release/mejorante evolve --steps 120 --spawn ./hijo --build
./hijo/target/debug/mejorante identity
```

`identity` imprime generación, cerebro y puntaje.  
`evolve --spawn ./hijo --build` busca un cerebro mejor, escribe un crate hijo en `./hijo` y lo compila.

## Comandos

```
mejorante identity              generación, linaje, cerebro, puntaje
mejorante eval [x]              cerebro vs la función objetivo
mejorante evolve                busca un cerebro mejor
                 --steps N      pasos de búsqueda (default 120)
                 --lambda L     mutantes por paso (default 30)
                 --seed S       rng reproducible
                 --spawn <dir>  hijo con el campeón
                 --build        compila a ese hijo
                 --force        pisa un hijo anterior
                 --write        pisa src/main.rs de este proyecto
mejorante dish                  anima una célula que encaja la curva
                 --steps N      pasos de búsqueda (default 120)
                 --lambda L     mutantes por paso (default 30)
                 --seed S       default 7 (la demo fiable)
                 --delay MS     ms por cuadro (default 80)
mejorante spawn <dir>           copia el genoma actual (sin buscar)
mejorante genome                imprime las fuentes embebidas
```

`--spawn` deja este programa en paz y escribe un hijo elegido.  
`--write` edita el `src/main.rs` de este proyecto; compilá de nuevo para que el binario nazca con el cerebro nuevo.

## Cómo funciona

El cerebro vive en una constante de `src/main.rs` en notación prefija: `x`, enteros chicos, `+`, `-`, `*`. Ejemplo: `(+ (* x x) 1)` significa `x² + 1`.

1. Parsea el cerebro actual a un árbol.
2. Cada paso arma varios mutantes al azar (cambia una hoja o un operador, crece o achica).
3. Se queda con el mutante de menor `sse + 0.01 × tamaño`.
4. Con `--spawn`, escribe un proyecto Cargo completo cuya fuente contiene ese cerebro. Cuando compilás al hijo, la mejora queda horneada.

La función de puntaje no cambia nunca. Cuando el cerebro encaja el objetivo, las identidades algebraicas achican la expresión (`(+ x x)` pasa a `(* 2 x)`, se caen ceros y unos) sin cambiar el encaje.

## Seguridad

- Un hijo por corrida. No hay bucles en segundo plano ni red.
- No escribe sobre el directorio home, `/`, `/usr`, `/etc`, ni el directorio en el que estás parado.
- `--force` sólo borra una carpeta que ya parece un proyecto `mejorante`.

## Relacionados

[replicante](https://github.com/PascualMacana/replicante) es un hermano que se copia y no mejora.  
[demostrante](https://github.com/PascualMacana/demostrante) es un hermano que sólo escribe una mejora afirmada si hay una prueba verificable.  
[reinante](https://github.com/PascualMacana/reinante) es un hermano que se sigue reescribiendo porque el objetivo mismo se mueve.  
[cruzante](https://github.com/PascualMacana/cruzante) es un hermano que se queda con los cruces del río que todavía eran legales.
