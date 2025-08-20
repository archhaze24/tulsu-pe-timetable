import { addMessages, init, getLocaleFromNavigator } from 'svelte-i18n'
import ru from '../../lib/locales/ru.json'

// Register Russian messages and initialize i18n
addMessages('ru', ru)

init({
  fallbackLocale: 'ru',
  initialLocale: getLocaleFromNavigator() || 'ru',
})


