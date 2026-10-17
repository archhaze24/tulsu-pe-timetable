import '@fontsource/nunito/latin-400.css'
import '@fontsource/nunito/latin-500.css'
import '@fontsource/nunito/latin-600.css'
import '@fontsource/nunito/latin-700.css'
import '@fontsource/nunito/cyrillic-400.css'
import '@fontsource/nunito/cyrillic-500.css'
import '@fontsource/nunito/cyrillic-600.css'
import '@fontsource/nunito/cyrillic-700.css'
import './style.css'
import './lib/i18n'
import { mount } from 'svelte'
import App from './App.svelte'

mount(App, { target: document.getElementById('app')! })
