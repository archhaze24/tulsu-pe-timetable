import {createSignal} from "solid-js";
import logo from "./assets/logo.svg";
import "./App.css";
import greet from "./api/greet.ts";
import count from "./api/count.ts";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");
  const [counter, setCounter] = createSignal(0);

  async function greetFunction() {
    const result = await greet({name: name()})
    if (result.ok) {
      setGreetMsg(result.data)
    } else {
      setGreetMsg(result.error)
    }
  }

  async function countFunction() {
    const result = await count({id: 1})
    if (result.ok) {
      setCounter(result.data)
    } else {
      setGreetMsg(result.error)
    }
  }

  return (
    <main class="container">
      <h1>Welcome to Tauri + Solid</h1>

      <div class="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo"/>
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo"/>
        </a>
        <a href="https://solidjs.com" target="_blank">
          <img src={logo} class="logo solid" alt="Solid logo"/>
        </a>
      </div>
      <p>Click on the Tauri, Vite, and Solid logos to learn more.</p>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greetFunction();
          countFunction();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg()} {counter()}</p>
    </main>
  );
}

export default App;
