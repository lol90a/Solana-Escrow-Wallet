export type Escrow = {
  version: "0.1.0";
  name: "escrow";
  instructions: [
    {
      name: "createEscrow";
      accounts: [
        { name: "buyer"; isMut: true; isSigner: true },
        { name: "escrowAccount"; isMut: true; isSigner: false },
        { name: "systemProgram"; isMut: false; isSigner: false }
      ];
      args: [
        { name: "seller"; type: "publicKey" },
        { name: "amount"; type: "u64" },
        { name: "escrowId"; type: "u64" }
      ];
    },
    {
      name: "releaseFunds";
      accounts: [
        { name: "buyer"; isMut: true; isSigner: true },
        { name: "seller"; isMut: true; isSigner: false },
        { name: "escrowAccount"; isMut: true; isSigner: false },
        { name: "systemProgram"; isMut: false; isSigner: false }
      ];
      args: [];
    },
    {
      name: "cancelEscrow";
      accounts: [
        { name: "buyer"; isMut: true; isSigner: true },
        { name: "escrowAccount"; isMut: true; isSigner: false },
        { name: "systemProgram"; isMut: false; isSigner: false }
      ];
      args: [];
    }
  ];
  accounts: [
    {
      name: "EscrowAccount";
      type: {
        kind: "struct";
        fields: [
          { name: "buyer"; type: "publicKey" },
          { name: "seller"; type: "publicKey" },
          { name: "amount"; type: "u64" },
          { name: "status"; type: { defined: "EscrowStatus" } },
          { name: "escrowId"; type: "u64" },
          { name: "bump"; type: "u8" }
        ];
      };
    }
  ];
  types: [
    {
      name: "EscrowStatus";
      type: {
        kind: "enum";
        variants: [
          { name: "Pending" },
          { name: "Completed" },
          { name: "Cancelled" }
        ];
      };
    }
  ];
  errors: [
    { code: 6000; name: "NotBuyer"; msg: "Only the buyer can perform this action" },
    { code: 6001; name: "NotPending"; msg: "Escrow is not in pending status" },
    { code: 6002; name: "InvalidAmount"; msg: "Amount must be greater than zero" }
  ];
};

export const IDL: Escrow = {
  version: "0.1.0",
  name: "escrow",
  instructions: [
    {
      name: "createEscrow",
      accounts: [
        { name: "buyer", isMut: true, isSigner: true },
        { name: "escrowAccount", isMut: true, isSigner: false },
        { name: "systemProgram", isMut: false, isSigner: false }
      ],
      args: [
        { name: "seller", type: "publicKey" },
        { name: "amount", type: "u64" },
        { name: "escrowId", type: "u64" }
      ]
    },
    {
      name: "releaseFunds",
      accounts: [
        { name: "buyer", isMut: true, isSigner: true },
        { name: "seller", isMut: true, isSigner: false },
        { name: "escrowAccount", isMut: true, isSigner: false },
        { name: "systemProgram", isMut: false, isSigner: false }
      ],
      args: []
    },
    {
      name: "cancelEscrow",
      accounts: [
        { name: "buyer", isMut: true, isSigner: true },
        { name: "escrowAccount", isMut: true, isSigner: false },
        { name: "systemProgram", isMut: false, isSigner: false }
      ],
      args: []
    }
  ],
  accounts: [
    {
      name: "EscrowAccount",
      type: {
        kind: "struct",
        fields: [
          { name: "buyer", type: "publicKey" },
          { name: "seller", type: "publicKey" },
          { name: "amount", type: "u64" },
          { name: "status", type: { defined: "EscrowStatus" } },
          { name: "escrowId", type: "u64" },
          { name: "bump", type: "u8" }
        ]
      }
    }
  ],
  types: [
    {
      name: "EscrowStatus",
      type: {
        kind: "enum",
        variants: [{ name: "Pending" }, { name: "Completed" }, { name: "Cancelled" }]
      }
    }
  ],
  errors: [
    { code: 6000, name: "NotBuyer", msg: "Only the buyer can perform this action" },
    { code: 6001, name: "NotPending", msg: "Escrow is not in pending status" },
    { code: 6002, name: "InvalidAmount", msg: "Amount must be greater than zero" }
  ]
};
