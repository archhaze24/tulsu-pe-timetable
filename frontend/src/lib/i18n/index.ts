import { addMessages, init, getLocaleFromNavigator } from 'svelte-i18n'
import ru from '../../lib/locales/ru.json'

addMessages('ru', ru)

init({
  fallbackLocale: 'ru',
  initialLocale: getLocaleFromNavigator() || 'ru',
})


