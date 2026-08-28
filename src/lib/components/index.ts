export interface ComponentInventoryItem {
  readonly name: string;
  readonly firstMilestone: 'M1' | 'M3' | 'M4';
}

export const componentInventory = [
  { name: 'SiteHeader', firstMilestone: 'M1' },
  { name: 'DemoBanner', firstMilestone: 'M1' },
  { name: 'DockButton', firstMilestone: 'M1' },
  { name: 'TextLink', firstMilestone: 'M1' },
  { name: 'Field', firstMilestone: 'M1' },
  { name: 'StatusStamp', firstMilestone: 'M1' },
  { name: 'FilterRail', firstMilestone: 'M1' },
  { name: 'PurchaseOrderRow', firstMilestone: 'M1' },
  { name: 'POManifest', firstMilestone: 'M1' },
  { name: 'QuantityStepper', firstMilestone: 'M1' },
  { name: 'ScanCapture', firstMilestone: 'M1' },
  { name: 'ConnectionStrip', firstMilestone: 'M1' },
  { name: 'AttachmentTray', firstMilestone: 'M1' },
  { name: 'DiscrepancyPanel', firstMilestone: 'M1' },
  { name: 'ReceiptTimeline', firstMilestone: 'M1' },
  { name: 'ImportMapTable', firstMilestone: 'M3' },
  { name: 'ExportMenu', firstMilestone: 'M1' },
  { name: 'ConfirmDialog', firstMilestone: 'M1' },
  { name: 'LiveNotice', firstMilestone: 'M1' },
  { name: 'StatePanel', firstMilestone: 'M1' },
] as const satisfies readonly ComponentInventoryItem[];
