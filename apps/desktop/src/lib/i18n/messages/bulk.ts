import { defineMessages } from '../core.svelte';

interface BulkMessages {
  refRate: string;
  closeTitle: string;
  offers: string;
  openSite: string;
  noOffers: string;
  cols: { price: string; stock: string; seller: string; age: string };
}

/** Painel de currency/fragmentos (bulk exchange). */
export const bulkMsgs = defineMessages<BulkMessages>({
  'pt-BR': {
    refRate: 'Taxa de referência',
    closeTitle: 'Fechar (Esc)',
    offers: 'Ofertas',
    openSite: 'Abrir no site',
    noOffers: 'Nenhuma oferta encontrada',
    cols: { price: 'Preço / un.', stock: 'Estoque', seller: 'Vendedor', age: 'Idade' },
  },
  en: {
    refRate: 'Reference rate',
    closeTitle: 'Close (Esc)',
    offers: 'Offers',
    openSite: 'Open on site',
    noOffers: 'No offers found',
    cols: { price: 'Price / unit', stock: 'Stock', seller: 'Seller', age: 'Age' },
  },
  es: {
    refRate: 'Tasa de referencia',
    closeTitle: 'Cerrar (Esc)',
    offers: 'Ofertas',
    openSite: 'Abrir en el sitio',
    noOffers: 'No se encontraron ofertas',
    cols: { price: 'Precio / ud.', stock: 'Stock', seller: 'Vendedor', age: 'Antigüedad' },
  },
  de: {
    refRate: 'Referenzkurs',
    closeTitle: 'Schließen (Esc)',
    offers: 'Angebote',
    openSite: 'Auf Seite öffnen',
    noOffers: 'Keine Angebote gefunden',
    cols: { price: 'Preis / Stk.', stock: 'Bestand', seller: 'Verkäufer', age: 'Alter' },
  },
  'zh-CN': {
    refRate: '参考汇率',
    closeTitle: '关闭 (Esc)',
    offers: '报价',
    openSite: '在网站打开',
    noOffers: '未找到报价',
    cols: { price: '单价', stock: '库存', seller: '卖家', age: '时间' },
  },
  ja: {
    refRate: '参考レート',
    closeTitle: '閉じる (Esc)',
    offers: 'オファー',
    openSite: 'サイトで開く',
    noOffers: 'オファーが見つかりません',
    cols: { price: '単価', stock: '在庫', seller: '出品者', age: '経過' },
  },
  ko: {
    refRate: '기준 시세',
    closeTitle: '닫기 (Esc)',
    offers: '제안',
    openSite: '사이트에서 열기',
    noOffers: '제안을 찾을 수 없습니다',
    cols: { price: '단가', stock: '재고', seller: '판매자', age: '경과' },
  },
});
