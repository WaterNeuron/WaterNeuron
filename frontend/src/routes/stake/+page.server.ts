import en from './lang/en.json';
import es from './lang/es.json';
import ru from './lang/ru.json';
import ja from './lang/ja.json';

const translationsMap = {
	en: en,
	es: es,
	ru: ru,
	ja: ja
};

export function load() {
	return translationsMap;
}
