import { addMessages, init } from 'svelte-i18n'
import ru from '../locales/ru.json'

addMessages('ru', ru)

init({
  fallbackLocale: 'ru',
  initialLocale: 'ru',
})
