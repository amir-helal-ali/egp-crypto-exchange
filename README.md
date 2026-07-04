# منصة الجنيه للعملات الرقمية

منصة تبادل عملات رقمية مغلقة الحلقة قائمة على **الجنيه المصري (EGP)**، تتضمن:
- ✅ تداول سبوت (Spot) مع محرك مطابقة في الذاكرة
- ✅ تداول العقود الآجلة (Futures) برافعة مالية تصل إلى 125x
- ✅ نظام تداول بين الأفراد (P2P) مع نظام ضمان (Escrow)
- ✅ تحكم شامل للأدمن في كل تفاصيل المنصة
- ✅ واجهة عربية كاملة RTL مع رسوم بيانية احترافية
- ✅ بدون polling — كل البيانات لحظية عبر WebSocket

> **تنبيه:** هذه نسخة مرجعية. تشغيل منصة فعلية يتطلب تراخيص قانونية، التزام AML/KYC، حلول حراسة، وتدقيقات أمنية.

---

## نظرة عامة على البنية

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          المضيف / البروكسي العكسي                            │
└─────────────────────────────────────────────────────────────────────────────┘
        │                                │                                  │
        ▼                                ▼                                  ▼
┌─────────────────────┐         ┌────────────────────┐         ┌─────────────────────┐
│ واجهة المستخدم       │         │ الخلفية Rust        │         │ واجهة الإدارة         │
│ SvelteKit :3000     │ ◀────▶  │ Axum :8080         │ ◀────▶  │ SvelteKit :3001      │
│ (عربية RTL)         │         │ (محرك مطابقة)       │         │ (عربية RTL)          │
└─────────────────────┘         └────────────────────┘         └─────────────────────┘
                                         │  │  │
                ┌────────────────────────┘  │  └───────────────────────┐
                ▼                            ▼                          ▼
       ┌─────────────────┐         ┌──────────────────┐       ┌──────────────────┐
       │ PostgreSQL 16   │         │ Redis 7          │       │ Binance WS       │
       │ مستخدمون/محافظ/ │         │ طوابير/Pub-Sub   │       │ تدفق الأسعار     │
       │ أوامر/صفقات/    │         │ + نظام الضمان    │       │ + قاطع دائرة     │
       │ عقود/P2P/عملات  │         │ + الكاش          │       │                  │
       └─────────────────┘         └──────────────────┘       └──────────────────┘
```

## المميزات

### تداول السبوت (Spot)
- محرك مطابقة في الذاكرة باستخدام `BTreeMap` (O(log n))
- أوامر محددة (Limit) وسوقية (Market)
- دفتر أوامر لحظي بعمق 50 مستوى
- رسم بياني شموع يابانية (Candlestick) + حجم

### العقود الآجلة (Futures)
- رافعة مالية من 1x إلى 125x
- نمط هامش معزول (Isolated) أو متقاطع (Cross)
- حساب سعر التصفية تلقائياً
- تحديث أسعار السوق كل ثانيتين
- تصفية تلقائية عند بلوغ سعر التصفية
- صفقات طويلة (Long) وقصيرة (Short)

### التداول بين الأفراد (P2P)
- إنشاء عروض شراء/بيع بالجنيه
- 8 طرق دفع مصرية: بنك، فودافون كاش، إنستا باي، فوري، اتصالات كاش، أورانج كاش، وي باي، إيداع نقدي
- نظام ضمان (Escrow): حجز تلقائي للعملات حتى تأكيد الدفع
- محادثة بين الأطراف داخل الصفقة
- مهلة زمنية قابلة للتخصيص (15-120 دقيقة)
- تقييمات بعد إكمال الصفقة

### التحكم الشامل للأدمن
- **العملات**: إضافة/تعديل/حذف/تفعيل عملة، تحكم في الدقة ورسوم السحب والشبكة
- **أزواج التداول**: إضافة/تعديل/حذف/تفعيل، تحكم في رسوم الصانع والمنفذ لكل زوج، تفعيل السبوت/العقود الآجلة
- **الإعدادات**: سعر الجنيه/الدولار، أقل إيداع/سحب
- **المستخدمون**: تعديل الحالة، مستوى التحقق، إيقاف/حظر
- **الإيداعات والسحوبات**: طابور يدوي مع موضع كل طلب
- **مراكز العقود الآجلة**: مراقبة كل المراكز + إغلاق إجباري
- **سجل التدقيق**: كل إجراء مكتوب بشكل دائم

### واجهات احترافية
- **عربية كاملة RTL** بخط Cairo
- **رسوم بيانية احترافية** باستخدام `lightweight-charts` (نفس مكتبة TradingView)
- **دعم 6 إطارات زمنية**: 1m, 5m, 15m, 1h, 4h, 1d
- **بدون polling**: كل البيانات عبر WebSocket فقط

---

## بنية المستودع

```
.
├── backend/                          # الخلفية Rust (Axum + Tokio + SQLx)
│   ├── Cargo.toml
│   ├── Dockerfile                    # متعدد المراحل + Distroless نهائي
│   ├── migrations/
│   │   ├── 20240101000001_init.sql   # المخطط الأساسي
│   │   └── 20240102000001_futures_p2p.sql  # العقود + P2P + العملات
│   └── src/
│       ├── main.rs                   # نقطة الدخول
│       ├── config.rs, error.rs       # الإعدادات والأخطاء
│       ├── models/mod.rs             # كل النماذج
│       ├── db/                       # استعلامات SQLx لكل جدول
│       │   ├── users.rs, wallets.rs, orders.rs, trades.rs
│       │   ├── manual_transactions.rs, settings.rs
│       │   ├── futures.rs, p2p.rs, currencies.rs
│       ├── auth/                     # JWT + Extractors
│       ├── matching_engine/          # محرك BTreeMap
│       ├── binance/                  # WS + قاطع دائرة
│       ├── redis/                    # طوابير + Pub/Sub
│       ├── services/                 # أزواج التداول + الرسوم
│       └── api/
│           ├── auth.rs, user.rs, trading.rs, wallet.rs, admin.rs, ws.rs
│           ├── futures.rs            # العقود الآجلة
│           ├── p2p.rs                # التداول بين الأفراد
│           └── settings.rs           # إدارة العملات والأزواج
├── frontend-user/                    # واجهة المستخدم (port 3000)
│   └── src/
│       ├── lib/
│       │   ├── i18n/                 # قاموس الترجمة العربية
│       │   ├── api.ts, types.ts, stores.ts, format.ts
│       │   └── components/
│       │       ├── CandlestickChart.svelte  # رسم الشموع الاحترافي
│       │       ├── OrderBook.svelte, OrderForm.svelte
│       │       └── RecentTrades.svelte, CircuitBanner.svelte
│       └── routes/
│           ├── +layout.svelte, +page.svelte  # لوحة التحكم
│           ├── login/, register/
│           ├── trade/[pair]/         # شاشة التداول
│           ├── futures/[pair]/       # العقود الآجلة
│           ├── p2p/, p2p/create/, p2p/[id]/  # P2P
│           ├── wallet/, history/
├── frontend-admin/                   # واجهة الإدارة (port 3001)
│   └── src/routes/
│       ├── +layout.svelte, +page.svelte
│       ├── login/, deposits/, withdrawals/, users/
│       ├── liquidity/, currencies/, pairs/
│       ├── futures-positions/, p2p-trades/
│       ├── orders/, trades/, audit/, settings/
├── docker-compose.yml                # تنسيق كل الخدمات
├── .env.example                      # نسخ قبل النشر
└── README.md
```

---

## قاعدة البيانات

### الجداول الأساسية (migration 1)
| الجدول | الوصف |
|---|---|
| `users` | المستخدمون مع الدور والحالة ومستوى التحقق |
| `wallets` | محافظ لكل مستخدم × عملة (رصيد + محجوز) |
| `orders` | الأوامر المحددة والسوقية |
| `trades` | الصفقات (Taker × Maker) |
| `manual_transactions` | إيداعات EGP + سحب العملات |
| `wallet_ledger` | سجل دائم لكل تغيير في الرصيد |
| `admin_audit_log` | سجل إجراءات المديرين |
| `settings` | إعدادات JSONB |

### الجداول الجديدة (migration 2)
| الجدول | الوصف |
|---|---|
| `futures_positions` | مراكز العقود الآجلة |
| `funding_payments` | مدفوعات التمويل |
| `liquidations` | سجل التصفيات |
| `futures_user_settings` | إعدادات المستخدم للعقود |
| `p2p_offers` | عروض التداول بين الأفراد |
| `p2p_trades` | صفقات P2P |
| `p2p_messages` | محادثات الصفقات |
| `p2p_reviews` | تقييمات المستخدمين |
| `currencies` | العملات المدعومة |
| `trading_pairs` | أزواج التداول مع الإعدادات |

---

## التشغيل السريع (Docker Compose)

### المتطلبات
- Docker 24+
- Docker Compose v2.20+

### الخطوات

```bash
# 1. استنساخ المستودع
git clone https://github.com/amir-helal-ali/egp-crypto-exchange.git
cd egp-crypto-exchange

# 2. توليد سر JWT قوي
export JWT_SECRET=$(openssl rand -hex 32)

# 3. تخصيص البيئة (اختياري)
cp .env.example .env

# 4. بناء وتشغيل كل الخدمات
docker compose up -d --build

# 5. مراقبة سجلات الخلفية
docker compose logs -f backend
```

بعد التشغيل:
- **واجهة المستخدم:** http://localhost:3000
- **واجهة الإدارة:** http://localhost:3001
- **حالة الخلفية:** http://localhost:8080/health

بيانات المدير الافتراضية (غيّرها فوراً في الإنتاج):
```
البريد: admin@egp-exchange.local
كلمة المرور: ChangeMe!Admin2024
```

---

## نقاط API

### عامة
| الطريقة | المسار | الوصف |
|---|---|---|
| GET | `/health` | فحص الحياة |
| GET | `/api/market/tickers` | أسعار كل الأزواج |
| GET | `/api/market/orderbook/:pair` | دفتر الأوامر |
| GET | `/api/market/trades/:pair` | آخر الصفقات |
| GET | `/api/market/circuit` | حالة قاطع الدائرة |
| WS | `/api/market/ws` | تدفق لحظي شامل |

### المصادقة
| الطريقة | المسار | الوصف |
|---|---|---|
| POST | `/api/auth/register` | إنشاء حساب |
| POST | `/api/auth/login` | تسجيل دخول |
| POST | `/api/auth/refresh` | تجديد الرمز |

### المستخدم
| الطريقة | المسار | الوصف |
|---|---|---|
| GET | `/api/user/me` | ملف المستخدم |
| GET | `/api/user/wallets` | المحافظ |
| GET/POST | `/api/user/orders` | الأوامر |
| DELETE | `/api/user/orders/:id` | إلغاء أمر |
| GET/POST | `/api/user/deposits` | الإيداعات |
| GET/POST | `/api/user/withdrawals` | السحوبات |

### العقود الآجلة
| الطريقة | المسار | الوصف |
|---|---|---|
| GET/POST | `/api/futures/positions` | قائمة/فتح مركز |
| POST | `/api/futures/positions/:id/close` | إغلاق مركز |

### التداول بين الأفراد
| الطريقة | المسار | الوصف |
|---|---|---|
| GET/POST | `/api/p2p/offers` | قائمة/إنشاء عرض |
| GET | `/api/p2p/offers/:id` | تفاصيل عرض |
| POST | `/api/p2p/trades` | بدء صفقة |
| POST | `/api/p2p/trades/:id/paid` | تأكيد الدفع |
| POST | `/api/p2p/trades/:id/release` | إطلاق العملات |
| POST | `/api/p2p/trades/:id/cancel` | إلغاء الصفقة |
| GET/POST | `/api/p2p/trades/:id/messages` | الرسائل |

### الإدارة
| الطريقة | المسار | الوصف |
|---|---|---|
| GET | `/api/admin/overview` | نظرة عامة |
| GET/POST | `/api/admin/currencies` | العملات |
| PUT/DELETE | `/api/admin/currencies/:id` | تعديل/حذف عملة |
| GET/POST | `/api/admin/pairs` | أزواج التداول |
| PUT/DELETE | `/api/admin/pairs/:id` | تعديل/حذف زوج |
| GET/PUT | `/api/admin/settings` | إعدادات النظام |
| GET | `/api/admin/users` | المستخدمون |
| GET | `/api/admin/manual_tx` | الإيداعات/السحوبات |
| POST | `/api/admin/manual_tx/:id/review` | مراجعة طلب |
| GET | `/api/admin/futures/positions` | كل المراكز |

---

## الأمان

1. **سر JWT** يجب أن يكون 32 حرفاً على الأقل
2. **كلمات مرور PostgreSQL و Redis** يجب تغييرها في الإنتاج
3. **CORS** مقيّد على أصلين فقط (واجهة المستخدم + الإدارة)
4. **قاطع الدائرة** يحمي المستخدمين عند انقطاع تدفق بينانس
5. **نظام الضمان** في P2P يحجز العملات تلقائياً
6. **سجل التدقيق** دائم لكل إجراءات المديرين
7. **واجهتان معزولتان** بالكامل (مستخدم + إدارة)
8. **لا بوابة دفع خارجية** - النظام مغلق الحلقة

---

## قائمة الإنتاج

- [ ] توليد JWT_SECRET و كلمات مرور قوية
- [ ] إعداد proxy عكسي (Caddy/Nginx) مع TLS
- [ ] نسخ احتياطية لـ PostgreSQL
- [ ] استمرارية Redis (AOF مفعّل)
- [ ] Rate limiting middleware
- [ ] مقاييس Prometheus
- [ ] `cargo audit` دوري
- [ ] `cargo clippy -- -D warnings` في CI
- [ ] تجميع السجلات (Sentry / Loki)
- [ ] 2FA للمديرين
- [ ] قائمة IP المسموحة للوحة الإدارة
- [ ] إشعارات بالبريد/الSMS
- [ ] تدقيق أمني خارجي

---

## الترخيص

خاص..contact المنشئ للحصول على التفاصيل.
