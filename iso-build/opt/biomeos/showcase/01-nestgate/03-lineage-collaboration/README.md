# 03 - Lineage Collaboration: Trust-Based Sharing

**Duration**: 4 minutes  
**Prerequisites**: BearDog for lineage, NestGate running

---

## Overview

Share data securely across your lineage of trusted devices without exposing to third parties.

**What it demonstrates**:
- Lineage-based access control
- Trust inheritance (genesis → children)
- Zero-knowledge sharing
- Granular permissions
- Audit trails

---

## Philosophy

> "Your family shouldn't need Facebook to share photos.  
>  Your team shouldn't need Google to collaborate.  
>  Your devices are your lineage. Trust is inherited."

---

## Run the Demo

```bash
cd showcase/01-nestgate/03-lineage-collaboration
./demo.sh
```

---

## Key Concepts

### Lineage Trust Model
```
Genesis Device (you)
  ├─ Laptop (child 1)
  ├─ Phone (child 2)
  └─ Tablet (child 3)

Genesis can share with ANY child
Children can share with siblings (if granted)
All share access to genesis data
```

### Permission Levels
- **Read**: View data
- **Write**: Modify data
- **Share**: Grant access to others
- **Admin**: Full control + revoke

---

## Demo Flow

1. Genesis creates document
2. Laptop requests access (lineage verified)
3. Access granted automatically
4. Phone tries to access (no lineage)
5. Access denied
6. Genesis grants phone lineage
7. Phone can now access

---

**Philosophy**: *"Trust your devices, not corporations."*

