export interface RouteMeta {
  title: string;
  description: string;
}

export function routeMeta(pathname: string): RouteMeta {
  if (pathname === '/') return {
    title: 'Intake Desk — check supplier deliveries',
    description: 'Check supplier deliveries against purchase orders and record shortages or damage before the van leaves.',
  };
  if (pathname === '/privacy') return {
    title: 'Privacy — Intake Desk',
    description: 'How the Intake Desk demo stores sample changes on this device and keeps them out of production systems.',
  };
  if (pathname === '/terms') return {
    title: 'Terms — Intake Desk',
    description: 'Terms for using the local Intake Desk sample receiving workflow.',
  };
  if (pathname.startsWith('/demo/discrepancies/')) return {
    title: 'Discrepancy — Intake Desk',
    description: 'Review the sample shortage and damaged-item record.',
  };
  if (pathname.startsWith('/demo/receipts/')) return {
    title: 'Receipt — Intake Desk',
    description: 'Review and export the finalized sample receipt and correction history.',
  };
  if (pathname.startsWith('/demo/receive/')) return {
    title: 'Receive delivery — Intake Desk',
    description: 'Count the sample delivery, classify differences, and finalize its receipt.',
  };
  if (pathname.startsWith('/demo/purchase-orders/')) return {
    title: 'Purchase order — Intake Desk',
    description: 'Review the sample Northline Bearings purchase order before receiving it.',
  };
  if (pathname === '/demo') return {
    title: 'Demo — Intake Desk',
    description: 'Try Intake Desk with a ready Northline Bearings purchase order. Sample changes stay on this device.',
  };
  return {
    title: 'Page not found — Intake Desk',
    description: 'This Intake Desk page does not exist. Return home or open the sample receiving desk.',
  };
}
