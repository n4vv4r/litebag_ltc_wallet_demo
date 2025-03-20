<h1 align="center" id="title">Litebag (demo)</h1>

<p id="description">Billetera de Litecoin hecha en Rust + GTK4. Es una demo por lo cual solo he publicado una versión que sí funciona pero que no tiene todas las funciones que tendría una billetera. (Enviar o ver transacciones o generar direcciones temporales).</p>

<p align="center"><img src="https://img.shields.io/badge/Litecoin-A6A9AA?style=for-the-badge&amp;logo=Litecoin&amp;logoColor=white" alt="shields"><img src="https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&amp;logo=sqlite&amp;logoColor=white" alt="shields"><img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&amp;logo=rust&amp;logoColor=white" alt="shields"><img src="https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&amp;logo=linux&amp;logoColor=black" alt="shields"></p>

<h2>Screenshots:</h2>

<img src="https://cdn.discordapp.com/attachments/1326741426683510797/1352276182439038986/CapturaFull_2025-03-19_22-24-22.png?ex=67dd6cd5&amp;is=67dc1b55&amp;hm=f9659ec5465174b02cb6a82336b2d7afeda2b1ec307b9220236b1b07164ad425&amp;" alt="project-screenshot" width="600">

  
  
<h2>Cosas que tiene:</h2>

Aqui tiene algunas features:

*   Recibir
*   Generar
*   API de Blockcypher
*   100% personalizable

<br><h2>Pasos para tener tu entorno de trabajo:</h2>

<p>1. Instalar Rust</p>

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh source $HOME/.cargo/env
```

<p>2. Dependencias GTK4</p>

```bash
sudo apt install -y libgtk-4-dev
```

<p>3. Crear el proyecto</p>

```bash
cargo new litebag
```

<p>4. Entrar</p>

```bash
cd litebag
```

<p>5. Editar cargo.toml</p>

```toml
[dependencies] gtk = { version = "0.7" package = "gtk4" features = ["v4_12"] }
```

<p>6. Compilar y ejecutar</p>

```bash
cargo build && cargo run
```

<h2>Licencia:</h2>

Esto está licenciado bajo AGPL-3.0
