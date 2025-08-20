<script lang="ts">
  import logo from './assets/images/logo-universal.png'
  import {GetConfig, GetConfigPath} from '../wailsjs/go/app_services/App.js'
  import type {config} from '../wailsjs/go/models'

  let resultText: string = "Загрузка конфигурации..."
  let configPath: string = ""
  let configData: config.Config | null = null

  async function loadConfig(): Promise<void> {
    try {
      // Загружаем конфигурацию
      const configResponse = await GetConfig()
      if (configResponse.error) {
        resultText = `Ошибка загрузки конфигурации: ${configResponse.error}`
        return
      }
      configData = configResponse.data
      
      // Получаем путь к конфигурации
      const pathResponse = await GetConfigPath()
      if (pathResponse.error) {
        configPath = `Ошибка получения пути: ${pathResponse.error}`
      } else {
        configPath = pathResponse.data
      }
      
      resultText = "Конфигурация загружена успешно!"
    } catch (error) {
      resultText = `Ошибка: ${error}`
    }
  }

  // Загружаем конфигурацию при старте
  loadConfig()
</script>

<main>
  <img alt="Wails logo" id="logo" src="{logo}">
  <div class="result" id="result">{resultText}</div>
  
  {#if configData}
    <div class="config-info">
      <h3>Информация о конфигурации:</h3>
      <p><strong>Путь к базе данных:</strong> {configData.dbPath}</p>
      <p><strong>Файл конфигурации:</strong> {configPath}</p>
    </div>
  {/if}
  
  <div class="input-box" id="input">
    <button class="btn" on:click={loadConfig}>Обновить конфигурацию</button>
  </div>
</main>

<style>

  #logo {
    display: block;
    width: 50%;
    height: 50%;
    margin: auto;
    padding: 10% 0 0;
    background-position: center;
    background-repeat: no-repeat;
    background-size: 100% 100%;
    background-origin: content-box;
  }

  .result {
    height: 20px;
    line-height: 20px;
    margin: 1.5rem auto;
  }

  .input-box .btn {
    width: 60px;
    height: 30px;
    line-height: 30px;
    border-radius: 3px;
    border: none;
    margin: 0 0 0 20px;
    padding: 0 8px;
    cursor: pointer;
  }

  .input-box .btn:hover {
    background-image: linear-gradient(to top, #cfd9df 0%, #e2ebf0 100%);
    color: #333333;
  }

  .input-box .input {
    border: none;
    border-radius: 3px;
    outline: none;
    height: 30px;
    line-height: 30px;
    padding: 0 10px;
    background-color: rgba(240, 240, 240, 1);
    -webkit-font-smoothing: antialiased;
  }

  .input-box .input:hover {
    border: none;
    background-color: rgba(255, 255, 255, 1);
  }

  .input-box .input:focus {
    border: none;
    background-color: rgba(255, 255, 255, 1);
  }

  .config-info {
    margin: 2rem auto;
    padding: 1rem;
    background-color: rgba(240, 240, 240, 0.8);
    border-radius: 8px;
    max-width: 600px;
  }

  .config-info h3 {
    margin: 0 0 1rem 0;
    color: #333;
  }

  .config-info p {
    margin: 0.5rem 0;
    color: #666;
  }

  .config-info strong {
    color: #333;
  }

</style>
