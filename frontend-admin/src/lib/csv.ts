// أدوات تصدير CSV
// CSV export utilities

/**
 * تصدير بيانات إلى ملف CSV
 */
export function exportToCSV(filename: string, headers: string[], rows: (string | number)[][]) {
    const csvContent = [
        headers.join(','),
        ...rows.map((row) =>
            row.map((cell) => {
                const s = String(cell ?? '');
                // escape quotes and wrap in quotes if contains comma/quote/newline
                if (s.includes(',') || s.includes('"') || s.includes('\n')) {
                    return `"${s.replace(/"/g, '""')}"`;
                }
                return s;
            }).join(',')
        ),
    ].join('\n');

    // BOM for Arabic support in Excel
    const bom = '\uFEFF';
    const blob = new Blob([bom + csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = `${filename}_${new Date().toISOString().slice(0, 10)}.csv`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}
