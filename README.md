<h1 align="center">Reti</h1>
<div align="center">
Reti is an in-development calculator that can evaluate LaTeX expressions.
<br>
<img src="./docs/dark_icon.svg" width="300" />
</div>

<h2 align="center" id="development-status">Development Status</h2>

**Note:** Reti is currently under active development. For the latest and potentially unstable features, please check out the [`dev`](https://github.com/barbariand/Reti/tree/dev) branch.

- Current Phase: The project is actively under development with newer, experimental features being added in the dev branch.
- Stability: Users interested in the most up-to-date but possibly unstable builds should refer to the dev branch.

<h2 align="center">Table of Contents</h2>

- [Development Status](#development-status)
- [Installation](#installation)
- [REPL Usage](#repl-usage)
- [Features](#features)
- [Contribute](#contribute)
- [Contributors](#contributors)
- [Dependencies](#dependencies)
- [License](#license)

<h2 align="center" id="installation">Installation</h2>
To get started with Reti, clone the repository and build the project using Rust's package manager, Cargo. Follow these steps:

```bash
# Clone the repository
git clone https://github.com/barbariand/Reti.git

# Change directory to the repl
cd Reti/parser

# Build the project
cargo build --release
```

<h2 align="center" id="repl-usage">REPL Usage</h2>

Just input a string from any LaTeX mathematics and get back the calculated result

```
>> \displaystyle \frac{2\sqrt{9}+5}{3(3+4)+1}
> 0.5
```

The same LaTeX expression that was used as input can be rendered as $\displaystyle \frac{2\sqrt{9}+5}{3(3+4)+1}$. There is no need to translate between formats!

<h2 align="center" id="features">Features</h2>

- [ ] **Parsing Latex**: Parsing relevant mathematical LaTeX.
- [x] **REPL**: Evaluates a given mathematical expression in LaTeX and returns a result or as close as it could get.
- [ ] **Web API**: Server for running evaluations.
- [ ] **Web User Interface**: A website for easier use.

<h2 align="center" id="contribute">Contribute</h2>
Contributions are welcome! If you're interested in improving Reti, fork the repository, make your changes, and submit a pull request. We appreciate your input in making Reti better for everyone.

<h2 align="center" id="contributors">Contributors</h2>
Thanks to all the contributors who invest their time and expertise. Your efforts are greatly enhancing the usability of LaTeX in practical applications.

<h2 align="center" id="dependencies">Dependencies</h2>
RetiREPL relies on the following Rust dependencies:

- `tokio`
- `async-recursion`

The Reti app depends on the above and more(TBD).

<h2 align="center" id="license">License</h2>
The licensing details for this project are to be determined (TBD).
