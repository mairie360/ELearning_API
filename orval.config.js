module.exports = {
  mairie360: {
    input: './openapi.json',
    output: {
      mode: 'split',
      target: 'generated/endpoints',
      schemas: 'generated/model', // Met tous les types/interfaces ici
      client: 'axios', // ou 'fetch', 'axios' selon ton besoin
      mock: false,
    },
  },
};
